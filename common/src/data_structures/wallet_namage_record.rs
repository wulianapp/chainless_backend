use std::fmt;
use std::str::FromStr;

use super::secret_store::SecretStore;
use super::*;
use crate::env::CONF as global_conf;
use anyhow::Error;
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};

//const PREDECESSOR_SUBFIX: &str = ".node0";
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Eq, Hash, EnumString, ToString)]
pub enum WalletOperateType {
    //两个txid，服务端重试
    CreateAccount,
    //一个txid，服务端重试
    AddServant,
    //一个txid，服务端重试
    AddSubaccount,
    //一个txid，服务端重试
    RemoveServant,
    //一个txid，服务端重试
    UpdateStrategy,
    //一个txid，服务端重试
    UpdateSubaccountHoldLimit,
    //三个txid、用户重试
    ServantSwitchMaster,
    //三个txid、用户重试
    NewcomerSwitchMaster,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Eq, Hash)]
pub struct WalletManageRecord {
    pub record_id: String,
    pub user_id: String,
    pub operation_type: WalletOperateType,
    pub operator_pubkey: String,
    pub operator_device_id: String,
    pub operator_device_brand: String,
    pub tx_ids: Vec<String>,
    pub status: TxStatusOnChain,
}
