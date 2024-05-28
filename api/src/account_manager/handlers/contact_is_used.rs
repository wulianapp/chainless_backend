use common::error_code::{BackendError, BackendRes};

use models::account_manager::{UserFilter, UserInfoView};
use models::general::get_pg_pool_connect;
use models::{account_manager, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfoResponse {
    pub contact_is_register: bool,
    pub secruity_is_seted: bool,
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}

pub async fn req(request_data: ContactIsUsedRequest) -> BackendRes<UserInfoResponse> {
    let ContactIsUsedRequest { contact } = request_data;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let find_res =
        UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact), &mut db_cli).await;
    match find_res {
        Ok(info) => Ok(Some(UserInfoResponse {
            contact_is_register: true,
            secruity_is_seted: info.user_info.secruity_is_seted,
        })),
        Err(err) => {
            if err.to_string().contains("DataNotFound") {
                Ok(Some(UserInfoResponse {
                    contact_is_register: false,
                    secruity_is_seted: false,
                }))
            } else {
                Err(BackendError::InternalError(err.to_string()))
            }
        }
    }
}
