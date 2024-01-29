/***
success 0
account_manage api 1000
wallet api  2000
general api 3000
airdrop api 4000
newbie api 5000

error message is correspond with error code
*/

use std::fmt;
use near_primitives::types::AccountId;
use thiserror::Error;

#[derive(Debug)]
pub enum ApiCommonError {
    Unknown = 1000,
    RequestParamInvalid = 1001,
}

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("cann't find user's code in memory map")]
    UserVerificationCodeNotFound,
    #[error("user's code is expired")]
    UserVerificationCodeExpired,
    #[error("user's code is different with storage")]
    UserVerificationCodeIncorrect,
    #[error("user's phone number or email address is invalided")]
    PhoneOrEmailIncorrect,
    #[error("user's phone number or email already used for register")]
    PhoneOrEmailAlreadyRegister,
    #[error("user's phone number or email not register")]
    PhoneOrEmailNotRegister,
    #[error("user's password is incorrect")]
    PasswordIncorrect,
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("Captcha request too frequently")]
    CaptchaRequestTooFrequently,
    #[error("Account is locking")]
    AccountLocked,
    #[error("Invite code not exist")]
    InviteCodeNotExist,
}

impl ChainLessError for AccountManagerError {
    fn code(&self) -> u16 {
        match self {
            Self::InternalError(_string) => 2000,
            Self::InvalidParameters(_string) => 2001,
            Self::UserVerificationCodeNotFound => 2002,
            Self::UserVerificationCodeExpired => 2003,
            Self::UserVerificationCodeIncorrect => 2004,
            Self::PhoneOrEmailIncorrect => 2005,
            Self::PhoneOrEmailAlreadyRegister => 2006,
            Self::PhoneOrEmailNotRegister => 2008,
            Self::PasswordIncorrect => 2009,
            Self::Authorization(_string) => 2010,
            Self::CaptchaRequestTooFrequently => 2011,
            Self::AccountLocked => 2012,
            Self::InviteCodeNotExist => 2013,
        }
    }
}

#[derive(Debug)]
pub enum TxStatus {
    Success,
    Failed(String),
}

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("tx's from is not equal to user_id decoded from auth token")]
    TxFromNotBeUser,
    #[error("receiver is nonexistent  in database")]
    ReceiverNotFound,
    #[error("sender is nonexistent  in database")]
    SenderNotFound,
    #[error("Authorization error: {0}")]
    Authorization(String),
}
impl ChainLessError for WalletError {
    fn code(&self) -> u16 {
        match self {
            Self::Unknown(_String) => 3000,
            Self::TxFromNotBeUser => 3001,
            Self::ReceiverNotFound => 3002,
            Self::SenderNotFound => 3003,
            Self::Authorization(_String) => 3004,
        }
    }
}

pub trait ChainLessError {
    fn code(&self) -> u16;
}


