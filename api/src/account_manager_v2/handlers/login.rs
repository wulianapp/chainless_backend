use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use common::error_code::AccountManagerError::{self, AccountLocked, PasswordIncorrect};
use common::hash::Hash;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};

use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes};
use common::prelude::*;
use common::utils::time::now_millis;
use models::account_manager::UserFilter;
use models::{account_manager::UserInfoEntity, PsqlOp};
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref LOGIN_RETRY: Mutex<HashMap<u32, Vec<u64>>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    device_id: String,
    device_brand: String,
    contact: String,
    password: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginByCaptchaRequest {
    device_id: String,
    device_brand: String,
    contact: String,
    captcha: String,
}

fn record_once_retry(user_id: u32) -> Result<()> {
    let retry_storage = &mut LOGIN_RETRY
        .lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;
    let now = now_millis();
    retry_storage.entry(user_id).or_default().push(now);
    Ok(())
}

fn is_locked(user_id: u32) -> Result<(bool, u8, u64)> {
    let retry_storage = &mut LOGIN_RETRY
        .lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;
    let info = if let Some(records) = retry_storage.get(&user_id) {
        if records.len() >= LOGIN_BY_PASSWORD_RETRY_NUM as usize {
            let unlock_time = *records.last().unwrap() + LOGIN_UNLOCK_TIME;
            debug!("0002___{}", unlock_time);
            if now_millis() < unlock_time {
                (true, 0, unlock_time / 1000)
            } else {
                //clear retry records
                let _ = retry_storage.remove(&user_id);
                (false, 0, 0)
            }
        } else {
            (
                false,
                LOGIN_BY_PASSWORD_RETRY_NUM - 1 - records.len() as u8,
                0,
            )
        }
    } else {
        (false, LOGIN_BY_PASSWORD_RETRY_NUM - 1, 0)
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

    let user_info =
        UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
            .await
            .map_err(|e| {
                if e.to_string().contains("DBError::DataNotFound") {
                    AccountManagerError::PhoneOrEmailNotRegister.into()
                } else {
                    BackendError::InternalError(e.to_string())
                }
            })?
            .into_inner();

    let (is_lock, remain_chance, unlock_time) =
        is_locked(user_info.id).map_err(|e| BackendError::InternalError(e.to_string()))?;
    if is_lock {
        Err(AccountLocked(unlock_time))?;
    }

    if password.hash() != user_info.login_pwd_hash {
        record_once_retry(user_info.id)?;
        Err(PasswordIncorrect(remain_chance))?;
    } else {
        let retry_storage = &mut LOGIN_RETRY
            .lock()
            .map_err(|e| BackendError::InternalError(e.to_string()))?;

        let _ = retry_storage.remove(&user_info.id);
    }

    let device = DeviceInfoEntity::new_with_specified(&device_id, &device_brand, user_info.id);
    device
        .safe_insert(DeviceInfoFilter::ByDeviceUser(&device_id, &user_info.id))
        .await?;

    //generate auth token
    let token = token_auth::create_jwt(
        user_info.id,
        user_info.token_version,
        &device_id,
        &device_brand,
    )?;
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

    let user_info =
        UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
            .await
            .map_err(|e| {
                if e.to_string().contains("DBError::DataNotFound") {
                    AccountManagerError::PhoneOrEmailNotRegister.into()
                } else {
                    BackendError::InternalError(e.to_string())
                }
            })?
            .into_inner();

    Captcha::check_and_delete(&user_info.id.to_string(), &captcha, Usage::Login)?;

    let device = DeviceInfoEntity::new_with_specified(&device_id, &device_brand, user_info.id);
    device
        .safe_insert(DeviceInfoFilter::ByDeviceUser(&device_id, &user_info.id))
        .await?;

    //generate auth token
    let token = token_auth::create_jwt(
        user_info.id,
        user_info.token_version,
        &device_id,
        &device_brand,
    )?;
    //成功登陆删掉错误密码的限制
    let retry_storage = &mut LOGIN_RETRY
        .lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;

    let _ = retry_storage.remove(&user_info.id);
    Ok(Some(token))
}
