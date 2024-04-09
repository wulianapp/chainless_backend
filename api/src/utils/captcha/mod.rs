pub mod email;
pub mod phone;

use std::collections::HashMap;
use std::str::FromStr;

use common::error_code::{AccountManagerError, BackendError};
use common::utils::math::gen_random_verify_code;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::debug;

use common::env::ServiceMode;
use regex::Regex;

use common::error_code::AccountManagerError::*;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendError::*;

use common::utils::time::{now_millis, MINUTE10};
use phonenumber::Mode;
use strum_macros::{Display, EnumString, ToString};


lazy_static! {
    static ref CODE_STORAGE: Mutex<HashMap<(String, Usage), Captcha>> = Mutex::new(HashMap::new());
}

#[derive(PartialEq, Debug)]
pub enum ContactType {
    PhoneNumber,
    Email,
}

#[derive(PartialEq, Clone, Debug, Eq, Hash,EnumString,ToString)]
pub enum Usage {
    Register,
    Login,
    ResetLoginPassword,
    SetSecurity,
    //验证码有效期内只能发起一次转账
    PreSendMoney,
    PreSendMoneyToSub,
    PreSendMoneyToBridge,
    //SetSecurity,
    //AddServant,
    ServantSwitchMaster,
    NewcomerSwitchMaster,
}

/*** 
impl FromStr for Usage {
    type Err = BackendError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "register" => Ok(Self::Register),
            "resetPassword" => Ok(Self::ResetLoginPassword),
            "setSecurity" => Ok(Self::SetSecurity),
            "addServant" => Ok(Self::AddServant),
            "servantReplaceMaster" => Ok(Self::ServantSwitchMaster),
            "newcomerBecomeMaster" => Ok(Self::NewcomerSwitchMaster),
            _ => Err(RequestParamInvalid(s.to_string())),
        }
    }
}
*/

pub fn validate(input: &str) -> Result<ContactType, AccountManagerError> {
    // Updated regex for phone numbers with international dialing code
    if input.contains("@") {
        /*** 
        let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
        if email_re.is_match(input) {
            Ok(ContactType::Email)
        } else {
            Err(PhoneOrEmailIncorrect)
        }
        */
        if input.contains("@") {
            Ok(ContactType::Email)
        } else {
            Err(PhoneOrEmailIncorrect)
        }

    }else {
        //这里和前端的有效判断不一致先放开
        let number = phonenumber::parse(None, input);
        //if phonenumber::is_valid(&number){
        /***    
        if number.is_ok() {
            Ok(ContactType::PhoneNumber)
        } else {
            Err(PhoneOrEmailIncorrect)
        }
        */
        Ok(ContactType::PhoneNumber)
    }
}

pub fn get_captcha(user: String, kind: &Usage) -> Result<Option<Captcha>, BackendError> {
    debug!("get_captcha_find_{}_{}",user,kind.to_string());

    let code_storage = &CODE_STORAGE
        .lock()
        .map_err(|e| InternalError(e.to_string()))?;

        debug!("get_all_captcha {:?}",code_storage);
    let value = code_storage
        .get(&(user, kind.to_owned()))
        .as_ref()
        .map(|&x| x.to_owned());
    Ok(value)
}

#[derive(Debug, Clone)]
pub struct Captcha {
    //email address or phone number
    owner: String,
    device_id: String,
    kind: Usage,
    code: String,
    pub created_at: u64,
    pub expiration_time: u64,
}
//手机+852开头的后六位做验证码:  +86 13682470011
//邮箱test和@中间的字符，且字符长度等于6的作为验证码: test000001@gmail.com
//其他情况都是真随机验证码
pub fn distill_code_from_contact(contact: &str) -> String {
    if contact.contains("+86") {
        contact[contact.len() - 6..].to_string()
    } else if contact.parse::<u32>().is_ok(){
        //contact.to_string()
        "000000".to_string()
    }else {
        let re = Regex::new(r"test(.*?)@").unwrap();
        let mut code = gen_random_verify_code().to_string();
        if let Some(captures) = re.captures(contact) {
            if let Some(matched_text) = captures.get(1) {
                let filter_str = matched_text.as_str();
                if filter_str.len() == 6 {
                    code = filter_str.to_string();
                }
            }
        };
        code
    }
}

impl Captcha {
    pub fn new(user: String, device_id: String, kind: Usage) -> Self {
        let code = if common::env::CONF.service_mode != ServiceMode::Product
            && common::env::CONF.service_mode != ServiceMode::Dev
        {
            //distill_code_from_contact(&user)
            "000000".to_string()
        } else {
            gen_random_verify_code().to_string()
        };
        let now = now_millis();
        Captcha {
            owner: user,
            device_id,
            kind,
            code,
            created_at: now,
            expiration_time: now + MINUTE10,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expiration_time <= now_millis()
    }

    pub fn store(&self) -> Result<(), BackendError> {
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e| InternalError(e.to_string()))?;
        code_storage.insert((self.owner.to_string(), self.kind.clone()), self.clone());
        debug!("_eddy_store_{:?}",code_storage);

        Ok(())
    }

    pub fn delete(&self) -> Result<(), BackendError> {
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e| InternalError(e.to_string()))?;
        code_storage.remove(&(self.owner.to_string(), self.kind.clone()));
        Ok(())
    }

    pub fn check_user_code(user: &str, code: &str, kind: Usage) -> Result<(), BackendError> {
        if let Some(data) = get_captcha(user.to_string(), &kind)? {
            if data.code != code {
                Err(CaptchaIncorrect)?
            } else if data.code == code && data.is_expired() {
                Err(CaptchaExpired)?
            } else {
                //delete worn captcha
                let code_storage = &mut CODE_STORAGE
                .lock()
                .map_err(|e| InternalError(e.to_string()))?;
                code_storage.remove(&(user.to_string(), kind.clone()));
                Ok(())
            }
        } else {
            Err(CaptchaNotFound)?
        }
    }

    //todo: 验证验证码的时候不能进行验证码删除
    pub fn check_user_code2(user: &str, code: &str, kind: Usage) -> Result<(), BackendError> {
        if let Some(data) = get_captcha(user.to_string(), &kind)? {
            if data.code != code {
                Err(CaptchaIncorrect)?
            } else if data.code == code && data.is_expired() {
                Err(CaptchaExpired)?
            } else {
                Ok(())
            }
        } else {
            Err(CaptchaNotFound)?
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::captcha::email::send_email;
    use crate::utils::captcha::validate;
    use crate::utils::captcha::Captcha;
    use crate::utils::captcha::Usage;
    #[test]
    fn test_phone_valided() {
        assert!(validate("+86 13682471710").is_ok());
        assert!(validate("+355 88888888").is_ok());
        assert!(validate("+852 89587885").is_ok());
    }
}
