use actix_web::{Responder, web};
use log::info;
use serde::Serialize;
use common::error_code::{AccountManagerError, ApiError, WalletError};
use common::error_code::ApiCommonError::Internal;
use common::http::ApiRes;
use crate::account_manager::captcha::Captcha;
use crate::account_manager::{VerifyCodeRequest};

pub async fn req(request_data: VerifyCodeRequest) -> ApiRes<String> {
    let VerifyCodeRequest {
        device_id: _,
        user_contact,
        kind: _,
        captcha: code,
    } = request_data;

    //if user contact is invalided,it cann't store,and will return UserVerificationCodeNotFound in this func
    //let check_res = Captcha::check_user_code(&user_contact, &code)?;
    Err(Internal("".to_string()).into())?;
    Ok(None::<String>)
}