use actix_web::HttpRequest;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::{AccountManagerError, BackendError, BackendRes};

use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::utils::token_auth;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let token = crate::utils::token_auth::create_jwt(user_id, &device_id, &device_brand)?;
    Ok(Some(token))
}
