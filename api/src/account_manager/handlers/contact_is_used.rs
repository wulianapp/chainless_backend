use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoEntity};

use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContactIsUsedResponse {
    contact_is_register: bool,
    secruity_is_seted: bool,
}

pub async fn req(request_data: ContactIsUsedRequest) -> BackendRes<ContactIsUsedResponse> {
    let ContactIsUsedRequest { contact } = request_data;
    let find_res: Vec<UserInfoEntity> =
        UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact)).await?;
    Ok(Some(ContactIsUsedResponse {
        contact_is_register: !find_res.is_empty(),
        secruity_is_seted: true,
    }))
}
