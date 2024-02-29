pub mod account_manager;
pub mod airdrop;
pub mod general;
pub mod newbie_reward;
pub mod secret_store;
pub mod wallet;
pub mod device_info;

use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};
use crate::error_code::*;
use strum_macros::{EnumString, ToString,Display};


//only Main have key role
#[derive(Deserialize, Serialize, Debug,EnumString,Display)]
pub enum AccountKey {
    Main(KeyRole),
    Sub(String),
}

#[derive(Deserialize, Serialize, Debug,EnumString, Display,PartialEq)]
pub enum KeyRole {
    Master(String),
    Servant(String),
}

//never use it 
impl Default for KeyRole {
    fn default() -> Self {
      panic!("never use it ");
      Self::Master("".to_string())  
    }
}


#[derive(Deserialize, Serialize, Debug,EnumString,Display)]
pub enum SecretKeyType {
    SubaccountKey,
    MasterKey,
    ServantKey,
}

#[derive(Deserialize, Serialize, Debug,EnumString,Display,PartialEq)]
pub enum SecretKeyState {
    Sitting,
    Deprecated,
}

type DeviceType = Option<KeyRole>;


#[derive(Deserialize, Serialize, Debug,EnumString, Display,PartialEq)]
pub enum DeviceState {
    Active,
    Inactive,
}

