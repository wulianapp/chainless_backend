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
pub struct GenNewcomerSwitchMasterRequest {
    newcomer_pubkey: String,
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
    request_data: GenNewcomerSwitchMasterRequest,
) -> BackendRes<GenReplaceKeyResponse> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let GenNewcomerSwitchMasterRequest {
        newcomer_pubkey,
        captcha,
    } = request_data;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::NewcomerSwitchMaster)?;

    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Undefined)?;

    let client = ContractClient::<MultiSig>::new_query_cli().await?;
    let master_list = client.get_master_pubkey_list(&main_account).await?;

    if master_list.len() != 1 {
        return Err(BackendError::InternalError(
            "unnormal account,it's account have more than 1 master".to_string(),
        ));
    }

    let master = &master_list[0];

    let (add_key_txid, add_key_raw) = client.add_key(&main_account, &newcomer_pubkey).await?;
    let (delete_key_txid, delete_key_raw) = client.delete_key(&main_account, master).await?;
    let replace_txids = GenReplaceKeyResponse {
        add_key_txid,
        add_key_raw,
        delete_key_txid,
        delete_key_raw,
    };

    Ok(Some(replace_txids))
}
