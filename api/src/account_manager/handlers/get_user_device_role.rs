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
use crate::utils::{get_user_context, judge_role_by_user_id, token_auth};

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

    //针对用新设备查询
    let role = if  DeviceInfoEntity::find(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user.id),
         &mut db_cli).await?.is_empty(){
        KeyRole2::Undefined
    }else{
        get_user_context(&user.id,&device_id,&mut db_cli).await?.role()?
    };

    Ok(Some(role))
}
