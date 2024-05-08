use actix_web::HttpRequest;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::{AccountManagerError, BackendError, BackendRes};

use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::account_manager::{GetUserDeviceRoleRequest, UserInfoRequest};
use crate::utils::token_auth;

pub async fn req(request_data: GetUserDeviceRoleRequest) -> BackendRes<KeyRole2> {
    let GetUserDeviceRoleRequest { device_id, contact } = request_data;

    let user = account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))
        .map_err(|e| {
            if e.to_string().contains("DBError::DataNotFound") {
                AccountManagerError::PhoneOrEmailNotRegister.into()
            } else {
                BackendError::InternalError(e.to_string())
            }
        })?;

    if user.user_info.main_account.eq("") {
        return Ok(Some(KeyRole2::Undefined));
    }
    //todo:
    let find_res = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user.id));
    if let Err(err) = find_res {
        if err.to_string().contains("DBError::DataNotFound") {
            return Ok(Some(KeyRole2::Undefined));
        } else {
            return Err(BackendError::InternalError(err.to_string()));
        }
    }

    let (_, current_strategy, device) =
        crate::wallet::handlers::get_session_state(user.id, &device_id).await?;
    let role = crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
    Ok(Some(role))
}
