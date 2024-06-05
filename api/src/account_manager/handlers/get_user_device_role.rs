use actix_web::HttpRequest;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::{AccountManagerError, BackendError, BackendRes};

use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::{account_manager, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::utils::token_auth;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserDeviceRoleRequest {
    device_id: String,
    contact: String,
}

pub async fn req(request_data: GetUserDeviceRoleRequest) -> BackendRes<KeyRole2> {
    let GetUserDeviceRoleRequest { device_id, contact } = request_data;

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let user = account_manager::UserInfoEntity::find_single(
        UserFilter::ByPhoneOrEmail(&contact),
        &mut db_cli,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("DBError::DataNotFound") {
            AccountManagerError::PhoneOrEmailNotRegister.into()
        } else {
            BackendError::InternalError(e.to_string())
        }
    })?
    .into_inner();

    if user.main_account.is_none() {
        return Ok(Some(KeyRole2::Undefined));
    }
    //todo:
    let find_res = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user.id),
        &mut db_cli,
    )
    .await;
    if let Err(err) = find_res {
        if err.to_string().contains("DBError::DataNotFound") {
            return Ok(Some(KeyRole2::Undefined));
        } else {
            return Err(BackendError::InternalError(err.to_string()));
        }
    }

    let (_, current_strategy, device) =
        crate::wallet::handlers::get_session_state(user.id, &device_id, &mut db_cli).await?;
    let role = crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
    Ok(Some(role))
}
