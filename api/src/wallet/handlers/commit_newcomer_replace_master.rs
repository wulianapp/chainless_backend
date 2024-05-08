use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretStoreView, SecretUpdater};
use models::wallet_manage_record::WalletManageRecordView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{
    CommitNewcomerSwitchMasterRequest, CreateMainAccountRequest, ReconfirmSendMoneyRequest,
};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

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

    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Undefined)?;
    super::check_have_base_fee(&main_account).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;

    //get old_master
    let old_master = if master_list.len() == 1 {
        debug!("start switch servant to master");
        master_list[0].to_owned()
    } else if master_list.len() == 2 {
        warn!("unnormal account,it's account have 2 master");
        let mut local_list = master_list.clone();
        local_list.retain(|x| x.ne(&newcomer_pubkey));
        local_list[0].to_owned()
    } else {
        Err(BackendError::InternalError(
            "main account is unnormal".to_string(),
        ))?;
        unreachable!("");
    };

    models::general::transaction_begin()?;
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
            SecretStoreView::update_single(
                SecretUpdater::State(SecretKeyState::Incumbent),
                SecretFilter::ByPubkey(&newcomer_pubkey),
            )?;
        }

        //更新设备信息
        DeviceInfoView::update_single(
            DeviceInfoUpdater::BecomeMaster(&newcomer_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
        )?;
    } else {
        let err: String = format!("newcomer_pubkey<{}> already is master", newcomer_pubkey);
        Err(BackendError::InternalError(err))?;
    }

    //除了同时包含servant_key和旧的master之外的情况全部认为异常不处理
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;
    if master_list.len() == 2
        && master_list.contains(&newcomer_pubkey)
        && master_list.contains(&old_master)
    {
        blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig).await;
        //更新设备信息
        DeviceInfoView::update_single(
            DeviceInfoUpdater::BecomeUndefined(&old_master),
            DeviceInfoFilter::ByHoldKey(&old_master),
        )?;
    } else {
        Err(BackendError::InternalError(
            "main account is unnormal".to_string(),
        ))?;
    }

    let txid = multi_sig_cli
        .update_master(&main_account, newcomer_pubkey.clone())
        .await?;

    //前边两个用户管理的交互，可以无风险重试，暂时只有前两步完成，才能开始记录操作历史
    //从一开始就记录的话、状态管理太多
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::NewcomerSwitchMaster,
        &newcomer_pubkey,
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert()?;
    models::general::transaction_commit()?;
    Ok(None::<String>)
}
