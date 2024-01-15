pub mod email;
pub mod phone;

use std::collections::HashMap;

use common::error_code::AccountManagerError;
use common::utils::math::gen_random_verify_code;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::env::ServiceMode;
use regex::Regex;

const LIFETIME_SECONDS: u16 = 600; // 10 minutes

#[derive(PartialEq)]
pub enum ContactType {
    PhoneNumber,
    Email,
    Other,
}

pub fn validate(input: &str) -> ContactType {
    // Updated regex for phone numbers with international dialing code
    let phone_re = Regex::new(r"^\+\d{1,3}\s\d{10,15}$").unwrap();
    let email_re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();

    if phone_re.is_match(input) {
        ContactType::PhoneNumber
    } else if email_re.is_match(input) {
        ContactType::Email
    } else {
        ContactType::Other
    }
}

#[derive(Debug, Clone)]
pub struct VerificationCode {
    //email address or phone number
    owner: String,
    code: String,
    expiration_time: Instant,
}

impl VerificationCode {
    pub fn new(user: String) -> Self {
        let code = if common::env::CONF.service_mode == ServiceMode::Test {
            "000000".to_string()
        } else {
            gen_random_verify_code().to_string()
        };
        let expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS as u64);
        VerificationCode {
            owner: user,
            code,
            expiration_time,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expiration_time <= Instant::now()
    }

    pub fn store(&self) -> anyhow::Result<()> {
        let code_storage = &mut CODE_STORAGE.lock().unwrap();
        code_storage.insert(self.owner.to_string(), self.clone());
        Ok(())
    }

    pub fn check_user_code(user: &str, code: &str) -> Result<(), AccountManagerError> {
        let code_storage = &CODE_STORAGE.lock().unwrap();
        if let Some(data) = code_storage.get(user) {
            if data.code != code {
                Err(AccountManagerError::UserVerificationCodeIncorrect)
            } else if data.code == code && data.is_expired() {
                Err(AccountManagerError::UserVerificationCodeExpired)
            } else {
                Ok(())
            }
        } else {
            Err(AccountManagerError::UserVerificationCodeNotFound)
        }
    }
}

lazy_static! {
    static ref CODE_STORAGE: Mutex<HashMap<String, VerificationCode>> = Mutex::new(HashMap::new());
}
