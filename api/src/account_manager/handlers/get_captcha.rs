use actix_web::HttpRequest;
use common::data_structures::KeyRole;
//use log::debug;
use common::error_code::AccountManagerError::{self, CaptchaRequestTooFrequently};
use common::log::generate_trace_id;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};

use models::PsqlOp;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::utils::captcha::{email, Captcha, ContactType, Usage};
use crate::utils::captcha::{sms, Usage::*};
use crate::utils::{captcha, get_user_context, judge_role_by_user_id, token_auth};

use common::error_code::{BackendError, BackendRes, WalletError};
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
    
    let contract_type: ContactType = contact.parse()?;
    //兼容已登陆和未登陆
    let storage_key = match user_id {
        Some(id) => id.to_string(),
        None => contact.clone(),
    };

    if let Some(data) = captcha::get_captcha(&storage_key, &kind)? {
        let past_time = now_millis() - data.created_at;
        
        if past_time <= CAPTCHA_REQUEST_INTERVAL {
            let remain_time = CAPTCHA_REQUEST_INTERVAL - past_time;
            let remain_secs = (remain_time / 1000) as u8;
            Err(CaptchaRequestTooFrequently(remain_secs))?;
        } else if past_time <= CAPTCHA_EXPAIRE_TIME {
            debug!("send new code cover former code");
        } else {
            //delete old and regenerate new
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

    //delete expired captcha
    Captcha::clean_up_expired()?;
    Ok(None)
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

    //重置登录密码
    match kind {
        ResetLoginPassword => {
            let user_info = UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
                .await
                .map_err(|err| {
                    if err.to_string().contains("DBError::DataNotFound") {
                        AccountManagerError::PhoneOrEmailNotRegister.into()
                    } else {
                        BackendError::InternalError(err.to_string())
                    }
                })?
                .into_inner();
            let user_id = user_info.id;

            //安全问答之前都允许，安全问答之后只有主设备允许
            if user_info.main_account.is_some() {
                let find_device_res = DeviceInfoEntity::find_single(
                    DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
                )
                .await?
                .into_inner();
                let role =
                    judge_role_by_user_id(find_device_res.hold_pubkey.as_deref(), &user_id).await?;
                if role != KeyRole::Master {
                    Err(WalletError::UneligiableRole(role, KeyRole::Master))?;
                }
            }

            get(device_id, contact, kind, Some(user_info.id))
        }
        Register => {
            let find_res = UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact)).await;
            if find_res.is_ok() {
                Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
            }
            get(device_id, contact, kind, None)
        }
        Login => match UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact)).await {
            Ok(info) => get(device_id, contact, kind, Some(info.into_inner().id)),
            Err(err) => {
                if err.to_string().contains("DBError::DataNotFound") {
                    Err(AccountManagerError::PhoneOrEmailNotRegister)?
                } else {
                    Err(BackendError::InternalError(err.to_string()))?
                }
            }
        },
        SetSecurity | UpdateSecurity | ServantSwitchMaster | NewcomerSwitchMaster
        | ReplenishContact => Err(BackendError::RequestParamInvalid("".to_string()))?,
    }
}

pub async fn with_token_req(
    req: HttpRequest,
    request_data: GetCaptchaWithTokenRequest,
) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let GetCaptchaWithTokenRequest { contact, kind } = request_data;
    let kind: Usage = kind
        .parse()
        .map_err(|_err| BackendError::RequestParamInvalid(kind))?;
    let context = get_user_context(&user_id, &device_id).await?;
    let role = context.role()?;

    match kind {
        ResetLoginPassword | Register | Login => {
            Err(BackendError::RequestParamInvalid("".to_string()))?
        }
        SetSecurity | UpdateSecurity | ReplenishContact => {
            //要么为进行安全问答，否则就只能主设备
            if context.user_info.main_account.is_some() {
                if role != KeyRole::Master {
                    Err(WalletError::UneligiableRole(role, KeyRole::Master))?;
                }
            }
        }
        NewcomerSwitchMaster => {
            if role != KeyRole::Undefined {
                Err(WalletError::UneligiableRole(role, KeyRole::Undefined))?;
            }
        }
        ServantSwitchMaster => {
            if role != KeyRole::Servant {
                Err(WalletError::UneligiableRole(role, KeyRole::Servant))?;
            }
        }
    }

    get(device_id, contact, kind, Some(user_id))
}
