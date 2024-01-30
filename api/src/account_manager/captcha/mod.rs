pub mod email;
pub mod phone;

use std::collections::HashMap;
use std::str::FromStr;

use common::error_code::{AccountManagerError, ApiCommonError, ApiError};
use common::utils::math::gen_random_verify_code;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::env::ServiceMode;
use regex::Regex;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::AccountManagerError::*;
use common::error_code::ApiCommonError::*;
use common::http::ApiRes;
use common::utils::time::{MINUTE10, now_millis};

lazy_static! {
    static ref CODE_STORAGE: Mutex<HashMap<(String,Kind), Captcha >> = Mutex::new(HashMap::new());
}

#[derive(PartialEq)]
pub enum ContactType {
    PhoneNumber,
    Email,
}

#[derive(PartialEq,Clone,Debug,Eq,Hash)]
pub enum Kind {
    register,
    reset_password,
}

impl FromStr for Kind {
    type Err = ApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "register" => Ok(Self::register),
            "resetPassword" => Ok(Self::reset_password),
            _ => Err(RequestParamInvalid(s.to_string()).into()),
        }
    }
}

pub fn validate(input: &str) -> Result<ContactType,AccountManagerError> {
    // Updated regex for phone numbers with international dialing code
    let phone_re = Regex::new(r"^\+\d{1,3}\s\d{10,15}$").unwrap();
    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

    if phone_re.is_match(input) {
        Ok(ContactType::PhoneNumber)
    } else if email_re.is_match(input) {
        Ok(ContactType::Email)
    } else {
        Err(PhoneOrEmailIncorrect)
    }
}

pub fn validate_email(input: &str) -> Result<(),AccountManagerError> {
    // Updated regex for phone numbers with international dialing code


    let email_re =  Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

    if !email_re.is_match(input) {
        Err(PhoneOrEmailIncorrect)
    }else {
        Ok(())
    }
}

pub fn validate_phone(input: &str) -> Result<(),AccountManagerError> {
    // Updated regex for phone numbers with international dialing code
    let phone_re = Regex::new(r"^\+\d{1,3}\s\d{10,15}$").unwrap();

    if !phone_re.is_match(input) {
        Err(PhoneOrEmailIncorrect)
    }else {
        Ok(())
    }
}

pub fn get_captcha(user: String,kind:Kind) -> Option<Captcha> {
    let code_storage = &CODE_STORAGE.lock().unwrap();
    code_storage.get(&(user,kind)).as_ref().map(|&x| x.to_owned())
}

#[derive(Debug, Clone)]
pub struct Captcha {
    //email address or phone number
    owner: String,
    device_id: String,
    kind: Kind,
    code: String,
    pub created_at: u64,
    pub expiration_time: u64,
}

impl Captcha {
    pub fn new(user: String,device_id:String,kind: Kind) -> Self {
        let code = if common::env::CONF.service_mode != ServiceMode::Product
         && common::env::CONF.service_mode != ServiceMode::Dev {
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

    pub fn store(&self) -> Result<(),ApiError> {
        let code_storage = &mut CODE_STORAGE
            .lock()
            .map_err(|e|
                Internal(e.to_string()).into()
            )?;
        code_storage.insert((self.owner.to_string(),self.kind.clone()), self.clone());
        Ok(())
    }

    pub fn check_user_code(user: &str, code: &str,kind: Kind) -> Result<(), ApiError> {
        if let Some(data) = get_captcha(user.to_string(),kind) {
            if data.code != code {
                Err(UserVerificationCodeIncorrect.into())
            } else if data.code == code && data.is_expired() {
                Err(UserVerificationCodeExpired.into())
            } else {
                Ok(())
            }
        } else {
            Err(UserVerificationCodeNotFound.into())
        }
    }
}

