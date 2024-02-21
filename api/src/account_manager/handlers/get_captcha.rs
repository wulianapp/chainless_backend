//use log::debug;
use tracing::debug;
use common::error_code::AccountManagerError::CaptchaRequestTooFrequently;

use crate::account_manager::captcha::{Captcha, ContactType, Usage};
use crate::account_manager::{captcha, GetCaptchaRequest};
use common::http::BackendRes;
use common::utils::time::{now_millis, MINUTE1};

pub async fn req(request_data: GetCaptchaRequest) -> BackendRes<String> {
    let GetCaptchaRequest {
        device_id,
        contact,
        kind,
    } = request_data;
    let kind: Usage = kind.parse()?;
    //todo: only master device can reset password

    let contract_type = captcha::validate(&contact)?;
    if let Some(data) = captcha::get_captcha(contact.clone(), &kind)? {
        if now_millis() <= data.created_at + MINUTE1 {
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
