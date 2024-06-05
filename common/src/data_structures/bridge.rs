use super::*;
use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display as StdDisplay;
use std::str::FromStr;
use strum_macros::{Display, EnumString, ToString};

#[derive(Deserialize, Serialize, Debug, PartialEq, EnumString, Display, Clone)]
pub enum OrderType {
    Withdraw,
    Deposit,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum WithdrawStatus {
    /// 无链确认中
    ChainLessSigning,
    //ChainLessPending,
    //ChainLessFailed,
    /// 无链端成功   
    ChainLessSuccessful,
    /// 用户在外部链提现
    ExternalChainPending,
    /// 用户在外部链提现确认失败(回滚)
    ExternalChainFailed,
    /// 用户在外部链提现确认完毕
    ExternalChainConfirmed,
}

impl From<EthOrderStatus> for WithdrawStatus {
    fn from(value: EthOrderStatus) -> Self {
        match value {
            EthOrderStatus::Pending => Self::ExternalChainPending,
            EthOrderStatus::Failed => Self::ExternalChainFailed,
            EthOrderStatus::Confirmed => Self::ExternalChainConfirmed,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum DepositStatus {
    /// 用户在外部链提现
    ExternalChainPending,
    /// 用户在外部链提现确认失败(回滚)
    ExternalChainFailed,
    /// 用户在外部链提现确认完毕
    ExternalChainConfirmed,
    //后台直接查合约状态，不会有pending和failed
    //ChainLessPending,    /// 无链确认中
    //ChainLessFailed,     /// 无链端失败
    /// 无链端成功
    ChainLessSuccessful,
}

impl From<EthOrderStatus> for DepositStatus {
    fn from(value: EthOrderStatus) -> Self {
        match value {
            EthOrderStatus::Pending => Self::ExternalChainPending,
            EthOrderStatus::Failed => Self::ExternalChainFailed,
            EthOrderStatus::Confirmed => Self::ExternalChainConfirmed,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum EthOrderStatus {
    Pending,
    Failed,
    Confirmed,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct EthBridgeOrder {
    pub id: String,            //withdraw_id on chainless or deposit_id on ethereum
    pub order_type: OrderType, //Withdraw,Deposit
    pub chainless_acc: String, //无链id
    pub eth_addr: String,      //外链地址
    pub coin: CoinType,        //代币符号
    pub amount: u128,          //转账数量
    //pub status: String,            //订单状态
    pub status: EthOrderStatus, //WithdrawStatus,DepositStatus
    pub height: u64,
}
