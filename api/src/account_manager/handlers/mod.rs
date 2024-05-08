pub mod check_captcha;
pub mod contact_is_used;
pub mod get_captcha;
pub mod get_user_device_role;
pub mod login;
pub mod register;
pub mod reset_password;
pub mod user_info;

use common::{
    data_structures::account_manager::UserInfo,
    error_code::{AccountManagerError, BackendError, ExternalServiceError},
};
use models::{
    account_manager::{UserFilter, UserInfoView},
    PsqlOp,
};

/*****
fn get_user_info() -> Result<UserInfo,BackendError>{
    let find_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact));
    match find_res {
        Ok(info) => Ok(info.user_info),
        Err(err) => {
            if err.to_string().contains("DataNotFound") {
               Err(AccountManagerError::UserIdNotExist)
            } else if err.to_string().contains("RepeatedData") {
               Err(AccountManagerError::UserIdNotExist)
            } else {
                Err(ExternalServiceError::DBError(err.to_string()))
            }
        }
    }
}
**/
