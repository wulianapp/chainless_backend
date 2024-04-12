use actix_web::error::InternalError;
use actix_web::HttpRequest;
use common::data_structures::KeyRole2;
//use log::debug;
use common::error_code::AccountManagerError::{self, CaptchaRequestTooFrequently};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use tracing::{debug, info};

use crate::account_manager::{self, user_info, GetCaptchaWithoutTokenRequest, GetCaptchaWithTokenRequest};
use crate::utils::{captcha, token_auth};
use crate::utils::captcha::{email, Captcha, ContactType, Usage};
use common::error_code::{BackendError, BackendRes, ExternalServiceError, WalletError};
use common::utils::time::{now_millis, MINUTE1, MINUTE10};
use crate::utils::captcha::Usage::*;

//老的接口暂时不动它
pub async fn req(request_data: GetCaptchaWithoutTokenRequest) -> BackendRes<String> {
    let GetCaptchaWithoutTokenRequest {
        device_id,
        contact,
        kind,
    } = request_data;
    let kind: Usage = kind.parse().map_err(
        |_err| BackendError::RequestParamInvalid("".to_string()))?;
    //todo: only master device can reset password

    let contract_type = captcha::validate(&contact)?;
    if let Some(data) = captcha::get_captcha(contact.clone(), &kind)? {
        let past_time = now_millis() - data.created_at;
        if past_time <= MINUTE1 {
            let remain_time = MINUTE1 - past_time;
            let remain_secs = (remain_time / 1000) as u8;
            Err(CaptchaRequestTooFrequently(remain_secs))?;
        } else if past_time <= MINUTE10 {
            debug!("send new code cover former code");
        } else {
            //delete and regenerate new captcha
            let _ = data.delete();
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



fn get(device_id:String,contact:String,kind:Usage,user_id:Option<u32>) -> BackendRes<String> {
    let contract_type = captcha::validate(&contact)?;
    //fixme:
    let contact2 = contact.clone();

    //兼容已登陆和未登陆
    let contact = match user_id{
        Some(id) => id.to_string(),
        None => contact,
    };

    //todo: only master device can reset password
    if let Some(data) = captcha::get_captcha(contact.clone(), &kind)? {
        let past_time = now_millis() - data.created_at;
        if past_time <= MINUTE1 {
            let remain_time = MINUTE1 - past_time;
            let remain_secs = (remain_time / 1000) as u8;
            Err(CaptchaRequestTooFrequently(remain_secs))?;
        } else if past_time <= MINUTE10 {
            debug!("send new code cover former code");
        } else {
            //delete and regenerate new captcha
            let _ = data.delete();
        }
    }

 
    let code = Captcha::new(contact, device_id, kind);

    if contract_type == ContactType::PhoneNumber {
       //phone::send_sms(&code).unwrap()
       Err(ExternalServiceError::PhoneCaptcha("Not support Phone nowadays".to_string()))?;
    } else {
       email::send_email(&code,contact2)?;
    };
    
    code.store()?;

    //todo: delete expired captcha，so as to avoid use too much memory
    debug!("send code {:?}", code);
    Ok(None::<String>)
}

pub fn without_token_req(request_data: GetCaptchaWithoutTokenRequest) -> BackendRes<String>{
    let GetCaptchaWithoutTokenRequest {
        device_id,
        contact,
        kind,
    } = request_data;
    let kind: Usage = kind.parse().map_err(
        |_err| BackendError::RequestParamInvalid("".to_string()))?;



    //重置登录密码
    match kind {
        ResetLoginPassword => {
            let find_user_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))?;
            let user_id = find_user_res.id;

            if find_user_res.user_info.secruity_is_seted {
                let find_device_res = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id));
                if find_device_res.is_ok() && find_device_res.as_ref().unwrap().device_info.key_role.to_owned() == KeyRole2::Master {
                   debug!("line {}",line!());
                }else if find_device_res.is_err(){
                    Err(WalletError::UneligiableRole(
                        KeyRole2::Undefined,
                        KeyRole2::Master,
                    ))?;
                }else {
                    Err(WalletError::UneligiableRole(
                        find_device_res.unwrap().device_info.key_role,
                        KeyRole2::Master,
                    ))?;
                }
                
            }
           
        
            get(device_id,contact,kind,Some(find_user_res.id))                
        },
        Register => {
            let find_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact));
            if find_res.is_ok() {
                Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
            }
            get(device_id,contact,kind,None)
        },
        Login => {
            let find_res = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact));
            if find_res.is_err() {
                Err(AccountManagerError::PhoneOrEmailNotRegister)?;
            }
            get(device_id,contact,kind,Some(find_res.unwrap().id))
        },
        SetSecurity| UpdateSecurity | PreSendMoney |PreSendMoneyToSub| PreSendMoneyToBridge| ServantSwitchMaster | NewcomerSwitchMaster => {
            Err(AccountManagerError::CaptchaUsageNotAllowed)?
        }
    }
}


pub fn with_token_req(request: HttpRequest,request_data: GetCaptchaWithTokenRequest) -> BackendRes<String>{
    let (user_id, device_id, _) = token_auth::validate_credentials2(&request)?;
    let GetCaptchaWithTokenRequest {
        contact,
        kind,
    } = request_data;
    let kind: Usage = kind.parse().map_err(
        |_err| BackendError::RequestParamInvalid("".to_string()))?;
    let user = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    
    match kind {
        ResetLoginPassword | Register | Login => {
            Err(AccountManagerError::CaptchaUsageNotAllowed)?;
        },
        //验证码有效期内只能发起一次转账
        PreSendMoney | PreSendMoneyToSub | PreSendMoneyToBridge  => {
            if device.device_info.key_role != KeyRole2::Master {
                Err(WalletError::UneligiableRole(
                    device.device_info.key_role,
                    KeyRole2::Master,
                ))?;
            }
        },
        SetSecurity | UpdateSecurity => {
            if user.user_info.secruity_is_seted {
                if device.device_info.key_role != KeyRole2::Master {
                    Err(WalletError::UneligiableRole(
                        device.device_info.key_role,
                        KeyRole2::Master,
                    ))?;
                }
            }else {
                //may be unnecessary
                if device.device_info.key_role != KeyRole2::Undefined {
                    Err(WalletError::UneligiableRole(
                        device.device_info.key_role,
                        KeyRole2::Undefined,
                    ))?;
                }
            }
        },
        NewcomerSwitchMaster => {
            if device.device_info.key_role != KeyRole2::Undefined {
                Err(WalletError::UneligiableRole(
                    device.device_info.key_role,
                    KeyRole2::Undefined,
                ))?;
            }
        },
        ServantSwitchMaster => {
            if device.device_info.key_role != KeyRole2::Servant {
                Err(WalletError::UneligiableRole(
                    device.device_info.key_role,
                    KeyRole2::Servant,
                ))?;
            }
        }
    }

    get(device_id,contact,kind,Some(user_id))
}

