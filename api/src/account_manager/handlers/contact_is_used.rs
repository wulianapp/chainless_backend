use common::error_code::{BackendError, BackendRes};

use models::account_manager::{UserFilter, UserInfoView};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
//use super::super::ContactIsUsedRequest;
use crate::account_manager::ContactIsUsedRequest;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSimpleInfo {
    pub contact_is_register: bool,
    pub secruity_is_seted: bool,
}

pub fn req(request_data: ContactIsUsedRequest) -> BackendRes<UserSimpleInfo> {
    let ContactIsUsedRequest { contact } = request_data;
    //let find_res = account_manager::UserInfoView::find(UserFilter::ByPhoneOrEmail(&contact))?;
    let find_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact));
    match find_res {
        Ok(info) => Ok(Some(UserSimpleInfo {
            contact_is_register: true,
            secruity_is_seted: info.user_info.secruity_is_seted,
        })),
        Err(err) => {
            if err.to_string().contains("data isn't existed") {
                Ok(Some(UserSimpleInfo {
                    contact_is_register: false,
                    secruity_is_seted: false,
                }))
            } else {
                Err(BackendError::DBError(err.to_string()))
            }
        }
    }
}
