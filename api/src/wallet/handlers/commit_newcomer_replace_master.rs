use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretStoreView, SecretUpdater};
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{
    CommitNewcomerSwitchMasterRequest, CreateMainAccountRequest,ReconfirmSendMoneyRequest,
};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use common::error_code::BackendError::ChainError;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: CommitNewcomerSwitchMasterRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let CommitNewcomerSwitchMasterRequest {
        newcomer_pubkey,
        add_key_raw,
        delete_key_raw,
        add_key_sig,
        delete_key_sig,
        newcomer_prikey_encrypted_by_password,
        newcomer_prikey_encrypted_by_answer,
    } = request_data;
    let main_account = super::get_main_account(user_id)?;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Undefined {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Undefined,
        ))?;
    }

    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;

    if master_list.len() != 1 {
        error!("unnormal account， it's account have more than 1 master");
        return Err(common::error_code::BackendError::InternalError(
            "".to_string(),
        ));
    }
    let old_master = master_list.first().unwrap();

    //增加之前判断是否有
    if !master_list.contains(&newcomer_pubkey.to_string()) {
        blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw, &add_key_sig).await;

        //check if stored already ,if not insert sercret_store or update
        let origin_secret = SecretStoreView::find(SecretFilter::ByPubkey(&newcomer_pubkey))?;
        if origin_secret.is_empty() {
            let secret_info = SecretStoreView::new_with_specified(
                &newcomer_pubkey,
                user_id,
                &newcomer_prikey_encrypted_by_password,
                &newcomer_prikey_encrypted_by_answer,
            );
            secret_info.insert()?;
        } else {
            SecretStoreView::update(
                SecretUpdater::State(SecretKeyState::Incumbent),
                SecretFilter::ByPubkey(&newcomer_pubkey),
            )?;
        }

        //更新设备信息
        DeviceInfoView::update(
            DeviceInfoUpdater::BecomeMaster(&newcomer_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
        )?;
    } else {
        error!("newcomer_pubkey<{}> already is master", newcomer_pubkey);
        Err(BackendError::InternalError(
            "newcomer_pubkey already is master".to_string(),
        ))?;
    }

    //除了同时包含servant_key和旧的master之外的情况全部认为异常不处理
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;
    if master_list.len() == 2
        && master_list.contains(&newcomer_pubkey)
        && master_list.contains(old_master)
    {
        blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig).await;
        //更新设备信息
        DeviceInfoView::update(
            DeviceInfoUpdater::BecomeUndefined(old_master),
            DeviceInfoFilter::ByHoldKey(old_master),
        )?;
    } else {
        error!("main account is unnormal");
        Err(BackendError::InternalError(
            "main account is unnormal".to_string(),
        ))?;
    }
    multi_sig_cli
    .update_master(&main_account,newcomer_pubkey)
    .await?;

    Ok(None::<String>)
}
