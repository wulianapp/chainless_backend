use actix_web::{Responder, web};
use log::info;
use serde::Serialize;
use common::error_code::{AccountManagerError, WalletError};
use common::http::ApiRes;
use crate::account_manager::captcha::Captcha;
use crate::account_manager::{VerifyCodeRequest};

pub async fn req(request_data: VerifyCodeRequest) -> ApiRes<String, AccountManagerError> {
    let VerifyCodeRequest {
        device_id: _,
        user_contact,
        kind: _,
        captcha: code,
    } = request_data;

    //if user contact is invalided,it cann't store,and will return UserVerificationCodeNotFound in this func
    //let check_res = Captcha::check_user_code(&user_contact, &code)?;
    Err(AccountManagerError::InternalError("".to_string()))?;
    Ok(None::<String>)
}