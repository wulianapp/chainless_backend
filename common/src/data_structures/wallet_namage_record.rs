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
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Eq, Hash, EnumString, Display)]
pub enum WalletOperateType {
    /// 创建储蓄账户，五个txid
    CreateAccount,
    /// 添加从设备
    AddServant,
    /// 新设备变成从设备
    NewcomerSwitchServant,
    /// 添加子账户
    AddSubaccount,
    /// 删除从设备
    RemoveServant,
    /// 删除子账户
    RemoveSubaccount,
    /// 设置手续费
    SetFeesPriority,
    /// 更新多签策略
    UpdateStrategy,
    /// 更新子设备持仓限制
    UpdateSubaccountHoldLimit,
    /// 从设备替换主设备，三个txid
    ServantSwitchMaster,
    /// 新设备替换主设备，三个txid
    NewcomerSwitchMaster,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Eq, Hash)]
pub struct WalletManageRecord {
    pub record_id: String,
    pub user_id: u32,
    pub operation_type: WalletOperateType,
    pub operator_pubkey: String,
    pub operator_device_id: String,
    pub operator_device_brand: String,
    pub tx_ids: Vec<String>,
    pub status: TxStatusOnChain,
}
