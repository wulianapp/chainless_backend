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

/***
     //code is uncorrect
            }else if data.code == code && data.is_expired() {
                // code is expired
            }else {
                return Ok(())
            }
        }else {
                //not found
*/
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
    PhoneOrEmailNotRegister= 2006,
    PasswordIncorrect = 2007,
}

impl fmt::Display for AccountManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AccountManagerError::UserVerificationCodeNotFound => "cann't find user's code in memory map",
            AccountManagerError::UserVerificationCodeExpired => "user's code is expired",
            AccountManagerError::UserVerificationCodeIncorrect => "user's code is different with storage",
            AccountManagerError::PhoneOrEmailIncorrect => "user's phone number or email address is invalided",
            AccountManagerError::PhoneOrEmailAlreadyRegister => "user's phone number or email already used for register",
            AccountManagerError::PhoneOrEmailNotRegister => "user's phone number or email not register",
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