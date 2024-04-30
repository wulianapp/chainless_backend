use std::str::FromStr;
use crate::{env::CONF as global_conf, error_code::*};
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};
use super::*;

#[derive(Deserialize, Serialize, Debug,PartialEq, Clone)]
pub enum ChainLessStatus {
    Default,
    Pending,
    Signed,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SignedOrder {
    #[serde(alias = "num")]
    pub eth_block_height: u64,
    pub signer: String,
    pub signature: Option<String>,
}



#[derive(Deserialize, Serialize, Debug,PartialEq, EnumString,Display,Clone)]
pub enum OrderType {
    Withdraw,
    Deposit,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum WithdrawStatus {
    ChainLessSended, //无链已转出
    ChainLessPending, //桥签名确认中
    ChainLessConfirmed, //桥签名确认完毕
    EthereumConfirmed,  //用户在eth端提现完毕
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum DepositStatus {
    EthereumConfirmed,  //eth端的桥合约到账
    ChainLessConfirmed, //无链端到账
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct EthBridgeOrder{
    pub id: String,                   //withdraw_id on chainless or deposit_id on ethereum        
    pub order_type: OrderType,     //Withdraw,Deposit
    pub chainless_acc: String,     //无链id
    pub eth_addr: String,           //外链地址
    pub coin: CoinType,            //代币符号
    pub amount: u128,              //转账数量
    //pub status: String,            //订单状态
    pub reserved_field1: String,   
    pub reserved_field2: String,            
    pub reserved_field3: String,                     
}