use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;

use crate::account_manager::VerifyCodeRequest;

pub async fn req(request_data: VerifyCodeRequest) -> BackendRes<String> {
    let VerifyCodeRequest {
        device_id: _,
        user_contact: _,
        kind: _,
        captcha: _code,
    } = request_data;

    //if user contact is invalided,it cann't store,and will return UserVerificationCodeNotFound in this func
    //let check_res = Captcha::check_user_code(&user_contact, &code)?;
    Err(InternalError("".to_string()))?;
    Ok(None::<String>)
}
