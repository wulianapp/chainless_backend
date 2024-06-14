pub mod email;
pub mod sms;

use std::collections::HashMap;
use std::str::FromStr;

use common::error_code::{BackendError};
use common::utils::math::random_num;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::debug;
use common::env::ServiceMode;
use regex::Regex;
use common::error_code::AccountManagerError::*;
use common::error_code::BackendError::InternalError;
use common::constants::*;
use common::utils::time::now_millis;

use strum_macros::{Display, EnumString};

lazy_static! {
    static ref CODE_STORAGE: Mutex<HashMap<(String, Usage), Captcha>> = Mutex::new(HashMap::new());
}

#[derive(PartialEq, Debug)]
pub enum ContactType {
    PhoneNumber,
    Email,
}

impl FromStr for ContactType {
    type Err = BackendError;

    //目前联系方式的合法性由前端保证，后端只做简单甄别
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('@') {
            Ok(ContactType::Email)
        } else if s.contains('+') {
            Ok(ContactType::PhoneNumber)
        } else {
            Err(BackendError::RequestParamInvalid(s.to_string()))
        }
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Hash, EnumString, Display)]
pub enum Usage {
    Register,
    Login,
    ResetLoginPassword,
    SetSecurity,
    UpdateSecurity,
    ReplenishContact,
    //验证码有效期内只能发起一次转账
    //PreSendMoney,
    //PreSendMoneyToSub,
    //PreSendMoneyToBridge,
    //SetSecurity,
    //AddServant,
    ServantSwitchMaster,
    NewcomerSwitchMaster,
}

pub fn get_captcha(user: &str, kind: &Usage) -> Result<Option<Captcha>, BackendError> {
    debug!("get_captcha_find_{}_{}", user, kind.to_string());

    let code_storage = &CODE_STORAGE
        .lock()
        .map_err(|e| InternalError(e.to_string()))?;

    debug!("get_all_captcha {:?}", code_storage);
    let value = code_storage
        .get(&(user.to_owned(), kind.to_owned()))
        .as_ref()
        .map(|&x| x.to_owned());
    Ok(value)
}

pub fn gen_random_verify_code() -> String {
    (random_num() % 900000 + 100000).to_string()
}

#[derive(Debug, Clone)]
pub struct Captcha {
    //email address or phone number
    owner: String,
    device_id: String,
    kind: Usage,
    pub code: String,
    pub created_at: u64,
    pub expiration_time: u64,
}
//手机+852开头的后六位做验证码:  +86 13682470011
//邮箱test和@中间的字符，且字符长度等于6的作为验证码: test000001@gmail.com
//其他情况都是真随机验证码
pub fn distill_code_from_contact(contact: &str) -> String {
    if contact.contains("+86") {
        contact[contact.len() - 6..].to_string()
    } else if contact.parse::<u32>().is_ok() {
        //contact.to_string()
        "000000".to_string()
    } else {
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
        let code = gen_random_verify_code();
        let now = now_millis();
        Captcha {
            owner: user,
            device_id,
            kind,
            code,
            created_at: now,
            expiration_time: now + CAPTCHA_EXPAIRE_TIME,
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
        debug!("_eddy_store_{:?}", code_storage);

        Ok(())
    }

    pub fn delete(&self) -> Result<(), BackendError> {
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e| InternalError(e.to_string()))?;
        code_storage.remove(&(self.owner.to_string(), self.kind.clone()));
        Ok(())
    }

    pub fn check(user: &str, code: &str, kind: Usage) -> Result<(), BackendError> {
        if common::env::CONF.service_mode != ServiceMode::Product
            && common::env::CONF.service_mode != ServiceMode::Dev
            && code.eq("000000")
        {
            return Ok(());
        }

        if let Some(data) = get_captcha(user, &kind)? {
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

    pub fn check_and_delete(user: &str, code: &str, kind: Usage) -> Result<(), BackendError> {
        Self::check(user, code, kind.clone())?;
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e| InternalError(e.to_string()))?;
        code_storage.remove(&(user.to_string(), kind));
        Ok(())
    }

    pub fn clean_up_expired() -> Result<(),BackendError>{
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e| InternalError(e.to_string()))?;
        
        code_storage.retain(|_k,v| {
            !v.is_expired()
        });
        Ok(())
    }
    //todo: restrict map size
}