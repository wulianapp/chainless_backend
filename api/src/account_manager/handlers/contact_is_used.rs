use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoEntity};

use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}

pub async fn req(request_data: ContactIsUsedRequest) -> BackendRes<bool> {
    let ContactIsUsedRequest { contact } = request_data;
    let find_res = UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact)).await?;
    Ok(Some(!find_res.is_empty()))
}
