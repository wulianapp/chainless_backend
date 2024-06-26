use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoEntity};

use models::PsqlOp;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContactIsUsedResponse {
    pub contact_is_register: bool,
    pub secruity_is_seted: bool,
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}

pub async fn req(request_data: ContactIsUsedRequest) -> BackendRes<ContactIsUsedResponse> {
    let ContactIsUsedRequest { contact } = request_data;
    let find_res = UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact)).await?;
    debug!("__________{:?}", find_res);
    if find_res.is_empty() {
        Ok(Some(ContactIsUsedResponse {
            contact_is_register: false,
            secruity_is_seted: false,
        }))
    } else {
        let secruity_is_seted = find_res[0].user_info.main_account.is_some();
        Ok(Some(ContactIsUsedResponse {
            contact_is_register: true,
            secruity_is_seted,
        }))
    }
}
