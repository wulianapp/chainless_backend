use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole, SecretKeyState};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::general::{get_pg_pool_connect, transaction_begin, transaction_commit};
use models::secret_store::{SecretFilter, SecretStoreEntity, SecretUpdater};
use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitNewcomerSwitchMasterRequest {
    newcomer_pubkey: String,
    add_key_raw: String,
    delete_key_raw: String,
    add_key_sig: String,
    delete_key_sig: String,
    newcomer_prikey_encrypted_by_password: String,
    newcomer_prikey_encrypted_by_answer: String,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: CommitNewcomerSwitchMasterRequest,
) -> BackendRes<String> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let CommitNewcomerSwitchMasterRequest {
        newcomer_pubkey,
        add_key_raw,
        delete_key_raw,
        add_key_sig,
        delete_key_sig,
        newcomer_prikey_encrypted_by_password,
        newcomer_prikey_encrypted_by_answer,
    } = request_data;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Undefined)?;
    super::check_have_base_fee(&main_account).await?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
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

    //增加之前判断是否有
    if !master_list.contains(&newcomer_pubkey.to_string()) {
        let _ =
            blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw, &add_key_sig).await;

        //check if stored already ,if not insert sercret_store or update
        let origin_secret =
            SecretStoreEntity::find(SecretFilter::ByPubkey(&newcomer_pubkey)).await?;
        if origin_secret.is_empty() {
            let secret_info = SecretStoreEntity::new_with_specified(
                &newcomer_pubkey,
                user_id,
                &newcomer_prikey_encrypted_by_password,
                &newcomer_prikey_encrypted_by_answer,
            );
            secret_info.insert().await?;
        } else {
            SecretStoreEntity::update_single(
                SecretUpdater::State(SecretKeyState::Incumbent),
                SecretFilter::ByPubkey(&newcomer_pubkey),
               
            )
            .await?;
        }

        //更新设备信息
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeMaster(&newcomer_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
           
        )
        .await?;
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
        let _ =
            blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig)
                .await;
        //更新设备信息
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeUndefined(&old_master),
            DeviceInfoFilter::ByHoldKey(&old_master),
           
        )
        .await?;
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
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::NewcomerSwitchMaster,
        &newcomer_pubkey,
        &context.device.id,
        &context.device.brand,
        vec![txid],
    );
    record.insert().await?;
    Ok(None::<String>)
}
