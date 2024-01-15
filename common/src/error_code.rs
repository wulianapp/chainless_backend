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
use thiserror::Error;

#[derive(Debug)]
pub enum ApiCommonError {
    Unknown = 1000,
    RequestParamInvalid = 1001,
}

#[derive(Debug)]
pub enum AccountManagerError {
    Unknown = 2000,
    UserVerificationCodeNotFound = 2001,
    UserVerificationCodeExpired = 2002,
    UserVerificationCodeIncorrect = 2003,
    PhoneOrEmailIncorrect = 2004,
    PhoneOrEmailAlreadyRegister = 2005,
    PhoneOrEmailNotRegister = 2006,
    PasswordIncorrect = 2007,
}

impl fmt::Display for AccountManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AccountManagerError::UserVerificationCodeNotFound => {
                "cann't find user's code in memory map"
            }
            AccountManagerError::UserVerificationCodeExpired => "user's code is expired",
            AccountManagerError::UserVerificationCodeIncorrect => {
                "user's code is different with storage"
            }
            AccountManagerError::PhoneOrEmailIncorrect => {
                "user's phone number or email address is invalided"
            }
            AccountManagerError::PhoneOrEmailAlreadyRegister => {
                "user's phone number or email already used for register"
            }
            AccountManagerError::PhoneOrEmailNotRegister => {
                "user's phone number or email not register"
            }
            AccountManagerError::PasswordIncorrect => "user's password is incorrect",
            AccountManagerError::Unknown => "unknown error",
        };
        write!(f, "{}", description)
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
impl WalletError {
    pub fn code(&self) -> u16 {
        match self {
            WalletError::Unknown(_String) => 3000,
            WalletError::TxFromNotBeUser => 3001,
            WalletError::ReceiverNotFound => 3002,
            WalletError::SenderNotFound => 3003,
            WalletError::Authorization(_String) => 3004,
        }
    }
}
