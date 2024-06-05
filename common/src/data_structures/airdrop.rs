use super::*;
use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use std::{clone, fmt::Display as StdDisplay};
use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Airdrop {
    pub user_id: u32,
    pub account_id: Option<String>,
    pub invite_code: String,
    pub predecessor_user_id: u32,
    pub predecessor_account_id: String,
    pub btc_address: Option<String>,
    pub btc_level: Option<u8>
}
