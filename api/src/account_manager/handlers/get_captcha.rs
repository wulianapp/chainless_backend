use actix_web::error::InternalError;
use actix_web::HttpRequest;
use common::data_structures::KeyRole2;
//use log::debug;
use common::error_code::AccountManagerError::{self, CaptchaRequestTooFrequently};
use common::log::generate_trace_id;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::utils::captcha::{email, Captcha, ContactType, Usage};
use crate::utils::captcha::{sms, Usage::*};
use crate::utils::{captcha, token_auth};
use common::env::CONF;
use common::error_code::{BackendError, BackendRes, ExternalServiceError, WalletError};
use common::prelude::*;
use common::utils::time::now_millis;

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptchaWithoutTokenRequest {
    device_id: String,
    contact: String,
    kind: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptchaWithTokenRequest {
    contact: String,
    kind: String,
}

fn get(
    device_id: String,
    contact: String,
    kind: Usage,
    user_id: Option<u32>,
) -> BackendRes<String> {
    //todo: 
    let contract_type = captcha::validate(&contact)?;
    //兼容已登陆和未登陆
    let storage_key = match user_id {
        Some(id) => id.to_string(),
        None => contact.clone(),
    };

    //todo: only master device can reset password
    if let Some(data) = captcha::get_captcha(&storage_key, &kind)? {
        let past_time = now_millis() - data.created_at;
        //todo:env
        if past_time <= CAPTCHA_REQUEST_INTERVAL {
            let remain_time = CAPTCHA_REQUEST_INTERVAL - past_time;
            let remain_secs = (remain_time / 1000) as u8;
            Err(CaptchaRequestTooFrequently(remain_secs))?;
        } else if past_time <= CAPTCHA_EXPAIRE_TIME {
            debug!("send new code cover former code");
        } else {
            //delete and regenerate new captcha
            //todo: unnecessary
            let _ = data.delete();
        }
    }

    let captcha = Captcha::new(storage_key, device_id, kind);
    captcha.store()?;

    let content = format!(
        "[ChainLess] Your captcha is: {}, valid for 10 minutes.",
        captcha.code
    );
    tokio::spawn(async move {
        let send_res = if contract_type == ContactType::PhoneNumber {
            let reference = generate_trace_id();
            sms::send_sms(&contact, &content, &reference).await
        } else {
            email::send_email(&contact, &content)
        };
        if let Err(e) = send_res {
            error!("send code({:?}) failed {}:", captcha, e.to_string());
        } else {
            debug!("send code successful {:?}", captcha);
        }
    });

    //todo: delete expired captcha，so as to avoid use too much memory
    Ok(None::<String>)
}

pub async fn without_token_req(request_data: GetCaptchaWithoutTokenRequest) -> BackendRes<String> {
    let GetCaptchaWithoutTokenRequest {
        device_id,
        contact,
        kind,
    } = request_data;
    let kind: Usage = kind
        .parse()
        .map_err(|_err| BackendError::RequestParamInvalid(kind))?;
    let mut db_cli = get_pg_pool_connect().await?;

    //重置登录密码
    match kind {
        ResetLoginPassword => {
            let user_info =
                UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact), &mut db_cli)
                    .await
                    .map_err(|err| {
                        if err.to_string().contains("DBError::DataNotFound") {
                            AccountManagerError::PhoneOrEmailNotRegister.into()
                        } else {
                            BackendError::InternalError(err.to_string())
                        }
                    })?.into_inner();
            let user_id = user_info.id;

            if user_info.main_account.is_some() {
                let find_device_res = DeviceInfoEntity::find_single(
                    DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
                    &mut db_cli,
                )
                .await;
                if find_device_res.is_ok()
                    && find_device_res.as_ref().unwrap().device_info.key_role == KeyRole2::Master
                {
                    debug!("line {}", line!());
                } else if find_device_res.is_err() {
                    //todo: return failed rep by error info
                    Err(WalletError::UneligiableRole(
                        KeyRole2::Undefined,
                        KeyRole2::Master,
                    ))?;
                } else {
                    Err(WalletError::UneligiableRole(
                        find_device_res.unwrap().device_info.key_role,
                        KeyRole2::Master,
                    ))?;
                }
            }

            get(device_id, contact, kind, Some(user_info.id))
        }
        Register => {
            let find_res =
                UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact), &mut db_cli)
                    .await;
            if find_res.is_ok() {
                Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
            }
            get(device_id, contact, kind, None)
        }
        Login => {
            match UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact), &mut db_cli)
                .await
            {
                Ok(info) => get(device_id, contact, kind, Some(info.into_inner().id)),
                Err(err) => {
                    if err.to_string().contains("DBError::DataNotFound") {
                        Err(AccountManagerError::PhoneOrEmailNotRegister)?
                    } else {
                        Err(BackendError::InternalError(err.to_string()))?
                    }
                }
            }
        }
        SetSecurity | UpdateSecurity | ServantSwitchMaster | NewcomerSwitchMaster | ReplenishContact => {
            Err(BackendError::RequestParamInvalid("".to_string()))?
        }
    }
}

pub async fn with_token_req(
    req: HttpRequest,
    request_data: GetCaptchaWithTokenRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let GetCaptchaWithTokenRequest { contact, kind } = request_data;
    let kind: Usage = kind
        .parse()
        .map_err(|_err| BackendError::RequestParamInvalid(kind))?;
    let mut db_cli = get_pg_pool_connect().await?;
    let user =
        UserInfoEntity::find_single(UserFilter::ById(&user_id), &mut db_cli).await?.into_inner();
    let device = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?
    .into_inner();

    match kind {
        ResetLoginPassword | Register | Login => {
            Err(BackendError::RequestParamInvalid("".to_string()))?
        }
        SetSecurity | UpdateSecurity | ReplenishContact => {
            if user.main_account.is_some() {
                if device.key_role != KeyRole2::Master {
                    Err(WalletError::UneligiableRole(
                        device.key_role,
                        KeyRole2::Master,
                    ))?;
                }
            } else {
                //may be unnecessary
                if device.key_role != KeyRole2::Undefined {
                    Err(WalletError::UneligiableRole(
                        device.key_role,
                        KeyRole2::Undefined,
                    ))?;
                }
            }
        }
        NewcomerSwitchMaster => {
            if device.key_role != KeyRole2::Undefined {
                Err(WalletError::UneligiableRole(
                    device.key_role,
                    KeyRole2::Undefined,
                ))?;
            }
        }
        ServantSwitchMaster => {
            if device.key_role != KeyRole2::Servant {
                Err(WalletError::UneligiableRole(
                    device.key_role,
                    KeyRole2::Servant,
                ))?;
            }
        }
    }

    get(device_id, contact, kind, Some(user_id))
}
