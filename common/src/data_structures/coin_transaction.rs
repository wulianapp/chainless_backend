use super::{CoinType, TxStatusOnChain};

use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum TxRole {
    #[strum(serialize = "Sender", to_string = "sender")]
    Sender,
    #[strum(serialize = "Receiver", to_string = "receiver")]
    Receiver,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum TxType {
    /// 普通交易
    Normal,
    /// 强制交易
    Forced,
    /// 跨链提现（储蓄账户给跨链桥转账）
    MainToBridge,
}

impl TxRole {
    pub fn counterparty(&self) -> Self {
        match self {
            TxRole::Sender => TxRole::Receiver,
            TxRole::Receiver => TxRole::Sender,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Serialize, Clone, EnumString, Display, Eq)]
pub enum CoinSendStage {
    /// 订单创建
    Created = 1,
    /// 从设备签名收集完毕
    SenderSigCompleted = 2,
    /// 接收方同意收款，只有nomarl类型的交易有该状态
    ReceiverApproved = 3,
    /// 接收方拒绝收款，只有nomarl类型的交易有该状态
    ReceiverRejected = 4,
    /// 发起方主设备取消转账
    SenderCanceled = 5,
    /// 发起方二次确认转账
    SenderReconfirmed = 6,
    /// 交易过期
    MultiSigExpired = 7,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CoinTransaction {
    pub order_id: String,
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub sender: String,   //uid
    pub receiver: String, //uid
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub stage: CoinSendStage,
    pub coin_tx_raw: String,
    pub signatures: Vec<String>,
    pub tx_type: TxType,
    pub chain_status: TxStatusOnChain,
    pub receiver_contact: Option<String>,
}
