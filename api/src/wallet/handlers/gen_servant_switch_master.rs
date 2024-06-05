use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::KeyRole2;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::secret_store::SecretStoreEntity;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{get_next_uid, UserFilter, UserInfoEntity, UserUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenServantSwitchMasterRequest {
    captcha: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenReplaceKeyResponse {
    pub add_key_txid: String,
    pub add_key_raw: String,
    pub delete_key_txid: String,
    pub delete_key_raw: String,
}
pub(crate) async fn req(
    req: HttpRequest,
    data: GenServantSwitchMasterRequest,
) -> BackendRes<GenReplaceKeyResponse> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials(&req)?;
    let GenServantSwitchMasterRequest { captcha } = data;
    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::ServantSwitchMaster)?;

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let servant_pubkey = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?
    .device_info
    .hold_pubkey
    .ok_or(BackendError::InternalError(
        "this haven't be servant yet".to_string(),
    ))?;

    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account.clone().unwrap();
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Servant)?;

    let client = ContractClient::<MultiSig>::new_query_cli().await?;
    let master_pubkey = client.get_master_pubkey(&main_account).await?;

    let (add_key_txid, add_key_raw) = client.add_key(&main_account, &servant_pubkey).await?;
    let (delete_key_txid, delete_key_raw) =
        client.delete_key(&main_account, &master_pubkey).await?;
    let replace_txids = GenReplaceKeyResponse {
        add_key_txid,
        add_key_raw,
        delete_key_txid,
        delete_key_raw,
    };
    Ok(Some(replace_txids))
}
