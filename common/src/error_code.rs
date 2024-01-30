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

#[derive(Error, Debug)]
pub enum ApiError{
    #[error("{0}")]
    Common(ApiCommonError),
    #[error("{0}")]
    AccountManager(AccountManagerError),
    #[error("{0}")]
    Wallet(WalletError)
}

impl ChainLessError for ApiError {
    fn code(&self) -> u16 {
        match self {
            ApiError::Common(err) => {err.code()}
            ApiError::AccountManager(err) => {err.code()}
            ApiError::Wallet(err) => {err.code()}
        }
    }
}

#[derive(Error, Debug)]
pub enum ApiCommonError {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("Request param is invalid: {0}")]
    RequestParamInvalid(String),
    #[error("Db error: {0}")]
    DB(String),
    #[error("Db error: {0}")]
    Chain(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
}

impl ChainLessError for ApiCommonError {
    fn code(&self) -> u16 {
        match self {
            Self::Internal(_) => 1,
            Self::RequestParamInvalid(_) => 2,
            Self::DB(_) => 3,
            Self::Chain(_) => 4,
            Self::Authorization(_) => 5,
        }
    }
}

impl Into<ApiError> for ApiCommonError{
    fn into(self) -> ApiError {
        crate::error_code::ApiError::Common(self)
    }
}

impl Into<ApiError> for WalletError{
    fn into(self) -> ApiError {
        crate::error_code::ApiError::Wallet(self)
    }
}

impl Into<ApiError> for AccountManagerError{
    fn into(self) -> ApiError {
        crate::error_code::ApiError::AccountManager(self)
    }
}


#[derive(Error, Debug)]
pub enum AccountManagerError {
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
            Self::UserVerificationCodeNotFound => 2002,
            Self::UserVerificationCodeExpired => 2003,
            Self::UserVerificationCodeIncorrect => 2004,
            Self::PhoneOrEmailIncorrect => 2005,
            Self::PhoneOrEmailAlreadyRegister => 2006,
            Self::PhoneOrEmailNotRegister => 2008,
            Self::PasswordIncorrect => 2009,
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
    #[error("tx's from is not equal to user_id decoded from auth token")]
    TxFromNotBeUser,
    #[error("receiver is nonexistent  in database")]
    ReceiverNotFound,
    #[error("sender is nonexistent  in database")]
    SenderNotFound,
}
impl ChainLessError for WalletError {
    fn code(&self) -> u16 {
        match self {
            Self::TxFromNotBeUser => 3001,
            Self::ReceiverNotFound => 3002,
            Self::SenderNotFound => 3003,
        }
    }
}

pub trait ChainLessError {
    fn code(&self) -> u16;
}

