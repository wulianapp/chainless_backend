use std::fmt;
use std::str::FromStr;

use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
pub struct SecretStore {
    pub account_id: String,
    pub user_id: u32,
    pub master_encrypted_prikey: String,
    pub servant_encrypted_prikeys: Vec<String>,
}