pub mod check_captcha;
pub mod contact_is_used;
pub mod gen_token;
pub mod get_captcha;
pub mod get_user_device_role;
pub mod login;
pub mod register;
pub mod replenish_contact;
pub mod reset_password;
pub mod user_info;

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
