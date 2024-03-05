//use log::debug;
use tracing::debug;
use common::error_code::AccountManagerError::CaptchaRequestTooFrequently;

use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::{captcha};
use crate::account_manager::GetCaptchaRequest;
use common::error_code::BackendRes;
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
        let remain_time = now_millis() - data.created_at;
        if remain_time <= MINUTE1 {
            let remain_secs = (remain_time / 1000) as u8;
            Err(CaptchaRequestTooFrequently(remain_secs))?;
        }else {
            //delete and regenerate new captcha
            data.delete();
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
