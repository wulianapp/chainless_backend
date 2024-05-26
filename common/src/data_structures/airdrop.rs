
use super::*;
use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display as StdDisplay;
use std::str::FromStr;
use strum_macros::{Display, EnumString, ToString};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Airdrop {
    pub user_id: String,
    pub account_id: Option<String>,   
    pub invite_code: String, 
    pub predecessor_user_id: String, 
    pub predecessor_account_id: Option<String>, 
    pub btc_address: Option<String>,      
    pub btc_level: Option<u8>,
    pub airdrop_reserved_field1: String,       
    pub airdrop_reserved_field2: String,
    pub airdrop_reserved_field3: String,
}