pub mod account_manager;
pub mod airdrop;
pub mod bridge;
pub mod coin_transaction;
pub mod device_info;
pub mod secret_store;
pub mod wallet_namage_record;

use std::str::FromStr;

use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use self::{coin_transaction::CoinTransaction, secret_store::SecretStore};

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum KeyRole {
    /// 主设备
    Master,
    /// 从设备
    Servant,
    /// 新设备
    Undefined,
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum SecretKeyState {
    /// 使用中
    Incumbent,
    /// 当一个设备被替换后，更新为此状态
    Abandoned,
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum DeviceState {
    Active,
    Inactive,
}

/***
 当进行一个操作的时候判断状态再看是否放行
 Idle 状态可以执行任何操作
 TransferBusy 只允许转账操作
 KeyManageBusy 不允许发起任何新的流程，
*/
#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum OpStatus {
    KeyManageBusy,
    TransferBusy,
    Idle,
}

pub fn get_support_coin_list() -> Vec<MT> {
    vec![
        MT::BTC,
        MT::ETH,
        MT::USDT,
        MT::USDC,
        MT::CLY,
        MT::DW20,
    ]
}

pub fn get_support_coin_list_without_cly() -> Vec<MT> {
    vec![
        MT::BTC,
        MT::ETH,
        MT::USDT,
        //CoinType::USDC,
        MT::DW20,
    ]
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq, Default)]
pub enum MT {
    #[default]
    #[strum(ascii_case_insensitive, to_string = "BTC")]
    BTC,
    #[strum(ascii_case_insensitive, to_string = "ETH")]
    ETH,
    #[strum(ascii_case_insensitive, to_string = "USDT")]
    USDT,
    #[strum(ascii_case_insensitive, to_string = "USDC")]
    USDC,
    #[strum(ascii_case_insensitive, to_string = "CLY")]
    CLY,
    #[strum(ascii_case_insensitive, to_string = "DW20")]
    DW20,
}

impl MT {
    pub fn erc20_ca(&self) -> Option<String> {
        match self {
            MT::BTC => Some(global_conf.eth_wbtc_contract.clone()),
            MT::ETH => None,
            MT::USDT => Some(global_conf.eth_usdt_contract.clone()),
            MT::USDC => Some(global_conf.eth_usdc_contract.clone()),
            MT::CLY => Some(global_conf.eth_cly_contract.clone()),
            MT::DW20 => None,
        }
    }

    //todo: config by env
    pub fn erc20_decimal(&self) -> Option<u8> {
        match self {
            MT::BTC => Some(18),
            //is token_decimal rather than coin_decimal
            MT::ETH => Some(18),
            MT::USDT => Some(18),
            MT::USDC => Some(18),
            MT::CLY => Some(18),
            MT::DW20 => None,
        }
    }

    pub fn mt_decimal(&self) -> u8 {
        match self {
            MT::BTC => 21,
            MT::ETH => 21,
            MT::USDT => 21,
            MT::USDC => 21,
            MT::CLY => 21,
            MT::DW20 => 21,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Display, Clone, Debug, Eq, Hash, EnumString)]
pub enum TxStatusOnChain {
    /// 创建好未广播
    NotLaunch,
    /// 已上链待确认
    Pending,
    /// 已上链但失败
    Failed,
    /// 已上链且成功
    Successful,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AccountMessage {
    pub newcomer_became_sevant: Vec<SecretStore>,
    pub coin_tx: Vec<CoinTransaction>,
    pub have_uncompleted_txs: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PubkeySignInfo {
    pub pubkey: String,
    pub signature: String,
}
impl FromStr for PubkeySignInfo {
    type Err = BackendError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //pubkey:64 sig:128
        if s.len() != 192 {
            Err(BackendError::RequestParamInvalid(s.to_string()))?;
        }
        Ok(PubkeySignInfo {
            pubkey: s[..64].to_string(),
            signature: s[64..].to_string(),
        })
    }
}
