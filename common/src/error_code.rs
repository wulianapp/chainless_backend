use thiserror::Error;

use crate::data_structures::KeyRole2;

pub type BackendRes<D, E = BackendError> = Result<Option<D>, E>;
use anyhow::Error as AnyhowError;


#[derive(Error, Debug)]
pub enum BackendError {
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("Request param is invalid: {0}")]
    RequestParamInvalid(String),
    #[error("Db error: {0}")]
    DBError(String),
    #[error("chain error: {0}")]
    ChainError(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("{0}")]
    ExternalService(#[from] ExternalServiceError),
    #[error("{0}")]
    AccountManager(#[from] AccountManagerError),
    #[error("{0}")]
    Wallet(#[from] WalletError),
}

impl From<AnyhowError> for BackendError {
    fn from(error: AnyhowError) -> Self {
        //db的错误可控，链的太多了
        if error.to_string().contains("DbError:") {
            BackendError::DBError(error.to_string())
        } else if error.to_string().contains("ChainError:") {
            BackendError::ChainError(error.to_string())
        }else {
            BackendError::InternalError(error.to_string())
        }
    }
}

impl ErrorCode for BackendError {
    fn code(&self) -> u16 {
        match self {
            BackendError::InternalError(_) => 1,
            BackendError::RequestParamInvalid(_) => 2,
            BackendError::DBError(_) => 3,
            BackendError::ChainError(_) => 4,
            BackendError::Authorization(_) => 5,
            BackendError::ExternalService(err) => err.code(),
            BackendError::AccountManager(err) => err.code(),
            BackendError::Wallet(err) => err.code(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error("cann't find user's code in memory map")]
    CaptchaNotFound,
    #[error("user's code is expired")]
    CaptchaExpired,
    #[error("user's code is different with storage")]
    CaptchaIncorrect,
    #[error("user's phone number or email address is invalided")]
    PhoneOrEmailIncorrect,
    #[error("user's phone number or email already used for register")]
    PhoneOrEmailAlreadyRegister,
    #[error("user's phone number or email not register")]
    PhoneOrEmailNotRegister,
    #[error("user's password is incorrect")]
    PasswordIncorrect,
    #[error("Captcha request too frequently,and it is alive in [{0}] secs")]
    CaptchaRequestTooFrequently(u8),
    #[error("Account is locking")]
    AccountLocked,
    #[error("Invite code not exist")]
    InviteCodeNotExist,
    #[error("user_id is not exist")]
    UserIdNotExist,
    #[error("CaptchaUsageNotAllowed")]
    CaptchaUsageNotAllowed,
}

impl ErrorCode for AccountManagerError {
    fn code(&self) -> u16 {
        match self {
            Self::CaptchaNotFound => 2002,
            Self::CaptchaExpired => 2003,
            Self::CaptchaIncorrect => 2004,
            Self::PhoneOrEmailIncorrect => 2005,
            Self::PhoneOrEmailAlreadyRegister => 2006,
            Self::PhoneOrEmailNotRegister => 2008,
            Self::PasswordIncorrect => 2009,
            Self::CaptchaRequestTooFrequently(_) => 2011,
            Self::AccountLocked => 2012,
            Self::InviteCodeNotExist => 2013,
            Self::UserIdNotExist => 2014,
            Self::CaptchaUsageNotAllowed => 2015,
        }
    }
}

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("tx's from is not equal to user_id decoded from auth token")]
    TxFromNotBeUser,
    #[error("receiver is nonexistent  in database")]
    ReceiverNotFound,
    #[error("sender is nonexistent  in database")]
    SenderNotFound,
    #[error("pubkey is not exist")]
    PubkeyNotExist,
    #[error("pubkey have already been exist")]
    PubkeyAlreadyExist,
    #[error("main_account {0} is not existent on chain")]
    MainAccountNotExist(String),
    #[error("have uncomplete transaction,cann't excute operate device")]
    HaveUncompleteTx,
    #[error("Current role is {0},but only {1} is allowed")]
    UneligiableRole(KeyRole2, KeyRole2),
    #[error("receiver is subaccount and trasfer value add balance value will exceed limit")]
    ExceedSubAccountHoldLimit,
    #[error("transfer amount big than available balance")]
    InsufficientAvailableBalance,
}
impl ErrorCode for WalletError {
    fn code(&self) -> u16 {
        match self {
            Self::TxFromNotBeUser => 3001,
            Self::ReceiverNotFound => 3002,
            Self::SenderNotFound => 3003,
            Self::PubkeyNotExist => 3004,
            Self::PubkeyAlreadyExist => 3005,
            Self::MainAccountNotExist(_) => 3006,
            Self::HaveUncompleteTx => 3007,
            Self::UneligiableRole(_, _) => 3008,
            Self::ExceedSubAccountHoldLimit => 3009,
            Self::InsufficientAvailableBalance => 3010,
        }
    }
}

#[derive(Error, Debug)]
pub enum ExternalServiceError {
    #[error("EmailCaptcha Service error: {0}")]
    EmailCaptcha(String),
    #[error("PhoneCaptcha Service error: {0}")]
    PhoneCaptcha(String),
    #[error("Database Service error: {0}")]
    Database(String),
    #[error("Chain Service error: {0}")]
    Chain(String),
}

impl ErrorCode for ExternalServiceError {
    fn code(&self) -> u16 {
        match self {
            Self::EmailCaptcha(_) => 101,
            Self::PhoneCaptcha(_) => 102,
            Self::Database(_) => 103,
            Self::Chain(_) => 104,
        }
    }
}

pub trait ErrorCode {
    fn code(&self) -> u16;
}
