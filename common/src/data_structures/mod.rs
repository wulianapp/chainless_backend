pub mod account_manager;
pub mod airdrop;
pub mod device_info;
pub mod general;
pub mod newbie_reward;
pub mod secret_store;
pub mod wallet;

use std::str::FromStr;

use crate::error_code::*;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};

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

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, EnumString, Display, PartialEq, Clone)]
pub enum TxStatus {
    Pending,
    Failed,
    Successful,
}
