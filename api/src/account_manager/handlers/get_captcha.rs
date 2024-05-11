use actix_web::error::InternalError;
use actix_web::HttpRequest;
use common::data_structures::KeyRole2;
//use log::debug;
use common::error_code::AccountManagerError::{self, CaptchaRequestTooFrequently};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use tracing::{debug, info,error};

use crate::account_manager::{
    self, user_info, GetCaptchaWithTokenRequest, GetCaptchaWithoutTokenRequest,
};
use crate::utils::captcha::Usage::*;
use crate::utils::captcha::{email, Captcha, ContactType, Usage};
use crate::utils::{captcha, token_auth};
use common::error_code::{BackendError, BackendRes, ExternalServiceError, WalletError};
use common::utils::time::{now_millis};
use common::env::CONF;
use common::prelude::*;

fn get(
    device_id: String,
    contact: String,
    kind: Usage,
    user_id: Option<u32>,
) -> BackendRes<String> {
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

    if contract_type == ContactType::PhoneNumber {
        Err(BackendError::InternalError(
            "Not support Phone nowadays".to_string(),
        ))?;
    } else {
        //缓存的key可能用的是user_id和contact，所以实际发送的邮箱地址需要额外参数提供
        //todo: 异步处理
        tokio::spawn(async move {
            if let Err(error) = email::send_email(&captcha.code, &contact){
                error!("send code failed {:?}", captcha);
            }else{
                debug!("send code successful {:?}", captcha);
            }
        });
    };

    //todo: delete expired captcha，so as to avoid use too much memory
    Ok(None::<String>)
}

pub fn without_token_req(request_data: GetCaptchaWithoutTokenRequest) -> BackendRes<String> {
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
            let find_user_res = UserInfoView::find_single(
                UserFilter::ByPhoneOrEmail(&contact)
            ).map_err(|err| {
                if err.to_string().contains("DBError::DataNotFound") {
                    AccountManagerError::PhoneOrEmailNotRegister.into()
                } else {
                    BackendError::InternalError(err.to_string())
                }
            })?;
            let user_id = find_user_res.id;

            if find_user_res.user_info.secruity_is_seted {
                let find_device_res = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(
                    &device_id, user_id,
                ));
                if find_device_res.is_ok()
                    && find_device_res
                        .as_ref()
                        .unwrap()
                        .device_info
                        .key_role
                        .to_owned()
                        == KeyRole2::Master
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

            get(device_id, contact, kind, Some(find_user_res.id))
        }
        Register => {
            let find_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact));
            if find_res.is_ok() {
                Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
            }
            get(device_id, contact, kind, None)
        }
        Login => match UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact)) {
            Ok(info) => get(device_id, contact, kind, Some(info.id)),
            Err(err) => {
                if err.to_string().contains("DBError::DataNotFound") {
                    Err(AccountManagerError::PhoneOrEmailNotRegister)?
                } else {
                    Err(BackendError::InternalError(err.to_string()))?
                }
            }
        },
        SetSecurity | UpdateSecurity | ServantSwitchMaster | NewcomerSwitchMaster => {
            Err(BackendError::RequestParamInvalid("".to_string()))?
        }
    }
}

pub fn with_token_req(
    req: HttpRequest,
    request_data: GetCaptchaWithTokenRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let GetCaptchaWithTokenRequest { contact, kind } = request_data;
    let kind: Usage = kind
        .parse()
        .map_err(|_err| BackendError::RequestParamInvalid(kind))?;
    let user = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;

    match kind {
        ResetLoginPassword | Register | Login => {
            Err(BackendError::RequestParamInvalid("".to_string()))?
        }
        SetSecurity | UpdateSecurity => {
            if user.user_info.secruity_is_seted {
                if device.device_info.key_role != KeyRole2::Master {
                    Err(WalletError::UneligiableRole(
                        device.device_info.key_role,
                        KeyRole2::Master,
                    ))?;
                }
            } else {
                //may be unnecessary
                if device.device_info.key_role != KeyRole2::Undefined {
                    Err(WalletError::UneligiableRole(
                        device.device_info.key_role,
                        KeyRole2::Undefined,
                    ))?;
                }
            }
        }
        NewcomerSwitchMaster => {
            if device.device_info.key_role != KeyRole2::Undefined {
                Err(WalletError::UneligiableRole(
                    device.device_info.key_role,
                    KeyRole2::Undefined,
                ))?;
            }
        }
        ServantSwitchMaster => {
            if device.device_info.key_role != KeyRole2::Servant {
                Err(WalletError::UneligiableRole(
                    device.device_info.key_role,
                    KeyRole2::Servant,
                ))?;
            }
        }
    }

    get(device_id, contact, kind, Some(user_id))
}
