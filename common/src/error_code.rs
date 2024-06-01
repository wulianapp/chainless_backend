use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};

use crate::{
    data_structures::{coin_transaction::CoinSendStage, KeyRole2},
    prelude::CoinType,
};
use serde::Deserialize;
use thiserror::Error;

pub type BackendRes<D, E = BackendError> = Result<Option<D>, E>;
use anyhow::Error as AnyhowError;
use serde_json;
use std::error::Error as StdError;
use strum_macros::{Display, EnumString};
use tracing::{debug, info};

#[derive(Deserialize, Debug, Default)]
pub struct MultiLangErrMsg {
    zh_cn: String,
    zh_tw: String,
    en_us: String,
}

lazy_static! {
    pub static ref ERR_CONF: HashMap<u16, MultiLangErrMsg> = {
        let path = &crate::env::CONF.error_code_path;
        let json_str = std::fs::read_to_string(path).unwrap();
        let err_code_map: HashMap<u16, MultiLangErrMsg> = serde_json::from_str(&json_str).unwrap();
        err_code_map
    };
}

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("Request param is invalid: {0}")]
    RequestParamInvalid(String),
    #[error("Check Signature failed")]
    SigVerifyFailed,
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
    #[error("{0}")]
    Bridge(#[from] BridgeError),
    #[error("{0}")]
    Airdrop(#[from] AirdropError),
}

impl From<AnyhowError> for BackendError {
    fn from(error: AnyhowError) -> Self {
        //组件的错误，全部为用户无关的
        BackendError::InternalError(error.to_string())
    }
}

pub fn to_internal_error<T: StdError>(error: T) -> BackendError {
    BackendError::InternalError(error.to_string())
}
pub fn to_param_invalid_error<T: StdError>(error: T) -> BackendError {
    BackendError::RequestParamInvalid(error.to_string())
}

pub fn parse_str<T, S>(data: S) -> Result<T, Box<dyn StdError>>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + StdError,
    S: Into<String>,
{
    let data: String = data.into();
    Ok(data.parse::<T>()?)
}

impl From<String> for BackendError {
    fn from(error: String) -> Self {
        BackendError::InternalError(error.to_string())
    }
}

impl From<&str> for BackendError {
    fn from(error: &str) -> Self {
        BackendError::InternalError(error.to_string())
    }
}

impl From<Box<dyn StdError>> for BackendError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        BackendError::InternalError(err.to_string())
    }
}

impl ErrorCode for BackendError {
    fn code(&self) -> u16 {
        match self {
            BackendError::InternalError(_) => 1,
            BackendError::RequestParamInvalid(_) => 2,
            BackendError::SigVerifyFailed => 3,
            BackendError::ChainError(_) => 4,
            BackendError::Authorization(_) => 5,
            BackendError::ExternalService(err) => err.code(),
            BackendError::AccountManager(err) => err.code(),
            BackendError::Wallet(err) => err.code(),
            BackendError::Bridge(err) => err.code(),
            BackendError::Airdrop(err) => err.code(),
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
    #[error("user's password is incorrect,remain [{0}] input chance")]
    PasswordIncorrect(u8),
    #[error("Captcha request too frequently,and it is alive in [{0}] secs")]
    CaptchaRequestTooFrequently(u8),
    #[error("Account is locking,unlock after timestamp [{0}]")]
    AccountLocked(u64),
    #[error("Invite code not exist")]
    InviteCodeNotExist,
    #[error("user_id is not exist")]
    UserIdNotExist,
    #[error("CaptchaUsageNotAllowed")]
    CaptchaUsageNotAllowed,
    #[error("PredecessorNotSetSecurity")]
    PredecessorNotSetSecurity,
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
            Self::PasswordIncorrect(_) => 2009,
            Self::CaptchaRequestTooFrequently(_) => 2011,
            Self::AccountLocked(_) => 2012,
            Self::InviteCodeNotExist => 2013,
            Self::UserIdNotExist => 2014,         //todo:interal
            Self::CaptchaUsageNotAllowed => 2015, //todo:params
            Self::PredecessorNotSetSecurity => 2016,
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
    #[error("NotSetSecurity")]
    NotSetSecurity,
    #[error("TxAlreadyConfirmed")]
    TxAlreadyConfirmed,
    #[error("Current status is {0},but only {1} is allowed")]
    TxStageIllegal(CoinSendStage, CoinSendStage),
    #[error("balanceMustBeZero")]
    BalanceMustBeZero,
    #[error("subaccount {0} is not existent on chain")]
    SubAccountNotExist(String),
    #[error("MustHaveSubaccount")]
    MustHaveSubaccount,
    #[error("ReceiverNotSetSecurity")]
    ReceiverNotSetSecurity,
    #[error("Receiver cann't be subaccount")]
    ReceiverIsSubaccount,
    #[error("Receiver must be subaccount")]
    ReceiverIsNotSubaccount,
    #[error("main_account {0} is already existent on chain")]
    MainAccountAlreadyExist(String),
    #[error("order_id {0} is nonexist")]
    OrderNotFound(String),
    #[error("Transfer amount cann't be zero")]
    FobidTransferZero,
    #[error("rank array of strategy is illegal")]
    StrategyRankIllegal,
    #[error("servant's num have already readch limit(11) ")]
    ServantNumReachLimit,
    #[error("transaction is already more than 24h")]
    TxExpired,
    #[error("Forbid Transfer to yourself")]
    ForbideTransferSelf,
    #[error("UnSupported Precision")]
    UnSupportedPrecision,
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
            Self::NotSetSecurity => 3011,
            Self::TxAlreadyConfirmed => 3012,
            Self::TxStageIllegal(_, _) => 3013,
            Self::BalanceMustBeZero => 3014,
            Self::SubAccountNotExist(_) => 3015,
            Self::MustHaveSubaccount => 3016,
            Self::ReceiverNotSetSecurity => 3017,
            Self::ReceiverIsSubaccount => 3018,
            Self::ReceiverIsNotSubaccount => 3019,
            Self::MainAccountAlreadyExist(_) => 3020,
            Self::OrderNotFound(_) => 3021,
            Self::FobidTransferZero => 3022,
            Self::StrategyRankIllegal => 3023,
            Self::ServantNumReachLimit => 3024,
            Self::TxExpired => 3025,
            Self::ForbideTransferSelf => 3026,
            Self::UnSupportedPrecision => 3027,
        }
    }
}

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Haven't set bind eth address")]
    NotBindEthAddr,
    #[error("{0} isn't supported cross transfer now")]
    CoinNotSupport(String),
}

impl ErrorCode for BridgeError {
    fn code(&self) -> u16 {
        match self {
            Self::NotBindEthAddr => 4000,
            Self::CoinNotSupport(_) => 4001,
        }
    }
}

#[derive(Error, Debug)]
pub enum AirdropError {
    #[error("Have bond a btc address")]
    AlreadyBindedBtcAddress,
    #[error("this address is already used")]
    BtcAddressAlreadyUsed,
    #[error("this invite code  is already used")]
    InviteCodeAlreadyUsed,
    #[error("this invite code  is illgae")]
    InviteCodeIllegal,
    #[error("this predecessor invite code  isn't  exist")]
    PredecessorInviteCodeNotExist,
    #[error("this predecessor cann't be your self")]
    ForbidSetSelfAsPredecessor,
}

impl ErrorCode for AirdropError {
    fn code(&self) -> u16 {
        match self {
            Self::AlreadyBindedBtcAddress => 5000,
            Self::BtcAddressAlreadyUsed => 5001,
            Self::InviteCodeAlreadyUsed => 5002,
            Self::InviteCodeIllegal => 5003,
            Self::PredecessorInviteCodeNotExist => 5004,
            Self::ForbidSetSelfAsPredecessor => 5005,
        }
    }
}

//不能设置自己为上级

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

pub trait ErrorCode: ToString {
    fn code(&self) -> u16;
    fn status_msg(&self, lang: LangType) -> String {
        match ERR_CONF.get(&self.code()) {
            Some(info) => match lang {
                LangType::ZH_TW => info.zh_tw.to_string(),
                LangType::ZH_CN => info.zh_cn.to_string(),
                LangType::EN_US => info.en_us.to_string(),
            },
            None => self.to_string(),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(EnumString, Display, PartialEq, Default)]
pub enum LangType {
    #[strum(ascii_case_insensitive)]
    ZH_TW,
    #[strum(ascii_case_insensitive)]
    ZH_CN,
    #[default]
    #[strum(ascii_case_insensitive)]
    EN_US,
}
