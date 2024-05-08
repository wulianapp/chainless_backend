use std::fmt;
use std::str::FromStr;

use super::{secret_store::SecretStore, CoinType, TxStatusOnChain};
use crate::env::CONF as global_conf;
use anyhow::Error;
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum TxRole {
    #[strum(serialize = "Sender", to_string = "sender")]
    Sender,
    #[strum(serialize = "Receiver", to_string = "receiver")]
    Receiver,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum TxType {
    Normal,
    Forced,
    MainToSub,
    SubToMain,
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
    Created = 1,
    SenderSigCompleted = 2,
    //子账户是接收者需要特殊对待
    //SenderSigCompletedAndReceiverIsSub,
    //SenderSigCompletedAndReceiverIsBridge,
    ReceiverApproved = 3, //只有nomarl类型的交易有该状态
    ReceiverRejected = 4, //只有nomarl类型的交易有该状态
    SenderCanceled = 5,
    SenderReconfirmed = 6,
    MultiSigExpired = 7, //从订单创建到上链结算的时间超时
                         //上链后，由于合约复杂度没有立即finalize
                         //ChainPending,
                         //如果commit之后没有finalize，更正Fail和Success的逻辑放在tx_list里面进行检查更新
                         //FinalizeAndFailed,
                         //FinalizeAndSuccessful,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CoinTransaction {
    pub order_id: String,
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub stage: CoinSendStage,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<String>,
    pub tx_type: TxType,
    pub chain_status: TxStatusOnChain,
    pub reserved_field2: String,
    pub reserved_field3: String,
}
