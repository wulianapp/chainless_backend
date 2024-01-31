use std::str::FromStr;
use std::time::{Duration, Instant};
use actix_web::{HttpResponse, Responder, web};
use log::{debug, info, trace};
use serde::Serialize;
use common::error_code::{AccountManagerError, ErrorCode, WalletError};
use common::error_code::AccountManagerError::CaptchaRequestTooFrequently;
use common::error_code::BackendError::{AccountManager};
use common::http::BackendRes;
use common::utils::time::{MINUTE1, MINUTE10, now_millis};
use crate::account_manager::{GetCaptchaRequest, captcha};
use crate::account_manager::captcha::{ContactType, Kind, Captcha};

pub async fn req(request_data: GetCaptchaRequest) -> BackendRes<String> {
    let GetCaptchaRequest { device_id, contact,kind} = request_data;
    let kind = Kind::from_str(&kind)?;
    //todo: only master device can reset password
    
    let contract_type = captcha::validate(&contact)?;
    if let Some(data) = captcha::get_captcha(contact.clone(),kind.clone())? {
        if now_millis() <= data.created_at + MINUTE1{
            Err(CaptchaRequestTooFrequently)?;
        }
    }

    let code = Captcha::new(contact, device_id, kind);
    if contract_type == ContactType::PhoneNumber {
        //phone::send_sms(&code).unwrap()
    } else {
        //email::send_email(&code).unwrap()
    };
    code.store()?;
    //todo: delete expired captcha，so as to avoid use too much memory
    debug!("send code {:?}", code);
    Ok(None::<String>)
}