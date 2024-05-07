use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::{Result};
use common::error_code::AccountManagerError::{
    self, AccountLocked, PasswordIncorrect, PhoneOrEmailNotRegister
};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use tracing::debug;

use crate::account_manager::{LoginByCaptchaRequest, LoginRequest};
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes};
use common::utils::time::{now_millis, MINUTE30};
use models::account_manager::UserFilter;
use models::{account_manager, PsqlOp};

lazy_static! {
    static ref LOGIN_RETRY: Mutex<HashMap<u32, Vec<u64>>> = Mutex::new(HashMap::new());
}

fn record_once_retry(user_id: u32) -> Result<()>{
    let retry_storage = &mut LOGIN_RETRY.lock()
    .map_err(|e| BackendError::InternalError(e.to_string()))?;
    let now = now_millis();
    debug!("0001___{}", now);
    retry_storage.entry(user_id).or_default().push(now);
    Ok(())
}

fn is_locked(user_id: u32) -> Result<(bool, u8, u64)> {
    let retry_storage = &mut LOGIN_RETRY.lock()
    .map_err(|e| BackendError::InternalError(e.to_string()))?;
    let info = if let Some(records) = retry_storage.get(&user_id) {
        if records.len() >= 5 {
            let unlock_time = *records.last().unwrap() + MINUTE30;
            debug!("0002___{}", unlock_time);
            if now_millis() < unlock_time {
                (true, 0, unlock_time as u64 / 1000)
            } else {
                //clear retry records
                let _ = retry_storage.remove(&user_id);
                (false, 0, 0)
            }
        } else {
            (false, 5 - 1 - records.len() as u8, 0)
        }
    } else {
        (false, 5 - 1, 0)
    };
    Ok(info)
}

pub async fn req_by_password(request_data: LoginRequest) -> BackendRes<String> {
    debug!("{:?}", request_data);
    let LoginRequest {
        device_id,
        device_brand,
        contact,
        password,
    } = request_data;
        
    let user_at_stored = account_manager::UserInfoView::find_single(
        UserFilter::ByPhoneOrEmail(&contact)
    ).map_err(|e| {
        if e.to_string().contains("DBError::DataNotFound") {
            AccountManagerError::PhoneOrEmailNotRegister.into()
        } else {
            BackendError::InternalError(e.to_string())
        }
    })?;


    let (is_lock, remain_chance, unlock_time) = is_locked(user_at_stored.id)
    .map_err(|e| BackendError::InternalError(e.to_string()))?;
    if is_lock {
        Err(AccountLocked(unlock_time))?;
    }

    if password != user_at_stored.user_info.login_pwd_hash {
        record_once_retry(user_at_stored.id)?;
        Err(PasswordIncorrect(remain_chance))?;
    } else {
        let retry_storage = &mut LOGIN_RETRY.lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;

        let _ = retry_storage.remove(&user_at_stored.id);
    }

    let device = DeviceInfoView::new_with_specified(&device_id, &device_brand, user_at_stored.id);
    device.safe_insert(DeviceInfoFilter::ByDeviceUser(
        &device_id,
        user_at_stored.id,
    ))?;

    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.id, &device_id, &device_brand)?;
    Ok(Some(token))
}

pub async fn req_by_captcha(request_data: LoginByCaptchaRequest) -> BackendRes<String> {
    debug!("{:?}", request_data);
    let LoginByCaptchaRequest {
        device_id,
        device_brand,
        contact,
        captcha,
    } = request_data;

    let user_at_stored = account_manager::UserInfoView::find_single(
        UserFilter::ByPhoneOrEmail(&contact)
    ).map_err(|e| {
        if e.to_string().contains("DBError::DataNotFound") {
            AccountManagerError::PhoneOrEmailNotRegister.into()
        } else {
            BackendError::InternalError(e.to_string())
        }
    })?;

    Captcha::check_user_code(&user_at_stored.id.to_string(), &captcha, Usage::Login)?;

    let device =
            DeviceInfoView::new_with_specified(&device_id, &device_brand, user_at_stored.id);
    device.safe_insert(DeviceInfoFilter::ByDeviceUser(
            &device_id,
            user_at_stored.id,
    ))?;


    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.id, &device_id, &device_brand)?;
    //成功登陆删掉错误密码的限制
    let retry_storage = &mut LOGIN_RETRY.lock()
    .map_err(|e| BackendError::InternalError(e.to_string()))?;

    let _ = retry_storage.remove(&user_at_stored.id);
    Ok(Some(token))
}
