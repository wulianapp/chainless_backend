pub mod account_manager;
pub mod airdrop;
pub mod coin_transaction;
pub mod device_info;
pub mod general;
pub mod newbie_reward;
pub mod secret_store;
pub mod wallet_namage_record;
pub mod bridge;

use std::str::FromStr;

use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};

use self::{coin_transaction::CoinTransaction, secret_store::SecretStore};

//only Main have key role
#[derive(Deserialize, Serialize, Debug, EnumString, Display)]
pub enum AccountKey {
    Main(KeyRole),
    Sub(String),
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq)]
pub enum KeyRole {
    Master(String),
    Servant(String),
    Newcommer(String),
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum KeyRole2 {
    Master,
    Servant,
    Undefined,
}

//never use it
impl Default for KeyRole {
    fn default() -> Self {
        panic!("never use it ");
        Self::Newcommer("".to_string())
    }
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display)]
pub enum SecretKeyType {
    SubaccountKey,
    MasterKey,
    ServantKey,
}

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum SecretKeyState {
    Incumbent,
    Abandoned,
}

type DeviceType = Option<KeyRole>;

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

pub fn get_support_coin_list() -> Vec<CoinType> {
    vec![
        CoinType::BTC,
        CoinType::ETH,
        CoinType::USDT,
        CoinType::USDC,
        CoinType::CLY,
        CoinType::DW20,
    ]
}

pub fn get_support_coin_list_without_cly() -> Vec<CoinType> {
    vec![
        CoinType::BTC,
        CoinType::ETH,
        CoinType::USDT,
        //CoinType::USDC,
        CoinType::DW20,
    ]
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq,Default)]
pub enum CoinType {
    #[default]
    #[strum(ascii_case_insensitive, to_string = "btc")]
    BTC,
    #[strum(ascii_case_insensitive, to_string = "eth")]
    ETH,
    #[strum(ascii_case_insensitive, to_string = "usdt")]
    USDT,
    #[strum(ascii_case_insensitive, to_string = "usdc")]
    USDC,
    #[strum(ascii_case_insensitive, to_string = "cly")]
    CLY,
    #[strum(ascii_case_insensitive, to_string = "dw20")]
    DW20,
}

impl CoinType {
    pub fn to_account_id(&self) -> AccountId {
        AccountId::from_str(&self.to_string()).unwrap()
    }

    pub fn erc20_ca(&self) -> Option<String> {
        match self {
            CoinType::BTC => Some(global_conf.eth_wbtc_contract.clone()),
            CoinType::ETH => None,
            CoinType::USDT => Some(global_conf.eth_usdt_contract.clone()),
            CoinType::USDC => Some(global_conf.eth_usdc_contract.clone()),
            CoinType::CLY => Some(global_conf.eth_cly_contract.clone()),
            CoinType::DW20 => None
        }
    }

    //todo: config by env
    pub fn erc20_decimal(&self) -> Option<u8> {
        match self {
            CoinType::BTC => Some(18),
            //is token_decimal rather than coin_decimal
            CoinType::ETH => Some(18),
            CoinType::USDT => Some(18),
            CoinType::USDC => Some(18),
            CoinType::CLY => Some(18),
            CoinType::DW20 => None,
        }
    }

    pub fn nep21_decimal(&self) -> u8 {
        match self {
            CoinType::BTC => 18,
            CoinType::ETH => 18,
            CoinType::USDT => 18,
            CoinType::USDC => 18,
            CoinType::CLY => 18,
            CoinType::DW20 => 18,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Display, Clone, Debug, Eq, Hash, EnumString)]
pub enum TxStatusOnChain {
    NotLaunch,
    Pending,
    Failed,
    Successful,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AccountMessage {
    pub newcomer_became_sevant: Vec<SecretStore>,
    pub coin_tx: Vec<CoinTransaction>,
    pub have_uncompleted_txs: bool,
}
