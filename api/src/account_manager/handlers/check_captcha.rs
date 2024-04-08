use common::error_code::{BackendError, BackendRes};

use models::account_manager::UserFilter;
use models::{account_manager, PsqlOp};
//use super::super::ContactIsUsedRequest;
use crate::account_manager::CheckCaptchaRequest;
use crate::utils::captcha::{Captcha, Usage};

pub fn req(request_data: CheckCaptchaRequest) -> BackendRes<bool> {
    let CheckCaptchaRequest { contact,captcha, usage} = request_data;
    let usage: Usage = usage.parse().map_err(
        |_err| BackendError::RequestParamInvalid("".to_string()))?;
    let user =
    account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))?;


    let check_res = Captcha::check_user_code(&user.id.to_string(), &captcha,usage);
    let is_ok = if check_res.is_err(){
        false
    }else{
        true
    };
    Ok(Some(is_ok))
}
