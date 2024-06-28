use common::data_structures::KeyRole;
use common::error_code::{AccountManagerError, BackendError, BackendRes};

use models::account_manager::UserFilter;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};

use crate::utils::get_user_context;
use models::{account_manager::UserInfoEntity, PsqlOp};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserDeviceRoleRequest {
    device_id: String,
    contact: String,
}

pub async fn req(request_data: GetUserDeviceRoleRequest) -> BackendRes<KeyRole> {
    let GetUserDeviceRoleRequest { device_id, contact } = request_data;

    /***
    let user = UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
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
    let role = if DeviceInfoEntity::find(DeviceInfoFilter::ByDeviceUser(&device_id, &user.id))
        .await?
        .is_empty()
    {
        KeyRole::Undefined
    } else {
        get_user_context(&user.id, &device_id).await?.role()?
    };
    ***/

    Ok(Some(KeyRole::Master))
}
