use std::fmt;
use std::str::FromStr;

use super::secret_store::SecretStore;
use anyhow::Error;
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};
use crate::env::CONF as global_conf;

const PREDECESSOR_SUBFIX: &str = ".node0";



//fixme: user_id is obsolate
/****
pub trait AddressConvert: Sized {
    fn to_user_id(&self) -> u32 {
        let id_str = self.to_account_str().replace(PREDECESSOR_SUBFIX, "");
        id_str.parse::<u32>().unwrap()
    }
    fn to_account_str(&self) -> String {
        self.to_user_id().to_string() + PREDECESSOR_SUBFIX
    }

    fn from_user_id(s: u32) -> Result<Self, String> {
        let new_s = s.to_string() + PREDECESSOR_SUBFIX;
        Self::from_account_str(&new_s)
    }

    fn from_account_str(s: &str) -> Result<Self, String> {
        let new_s = s.replace(PREDECESSOR_SUBFIX, "");
        let id = new_s.parse::<u32>().unwrap();
        Self::from_user_id(id)
    }
}
*/

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AccountMessage {
    NewcomerBecameSevant(SecretStore),
    CoinTx(u32, CoinTransaction2),
}

pub fn get_support_coin_list() -> Vec<CoinType> {
    vec![
        CoinType::BTC,
        CoinType::ETH,
        CoinType::USDT,
        CoinType::USDC,
        CoinType::CLY,
        CoinType::DW20,
    ]
}

pub fn get_support_coin_list_without_cly() -> Vec<CoinType> {
    vec![
        //CoinType::BTC,
        CoinType::ETH,
        CoinType::USDT,
        //CoinType::USDC,
        CoinType::DW20,
    ]
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display,PartialEq)]
pub enum CoinType {
    #[strum(ascii_case_insensitive,to_string = "btc")]
    BTC,
    #[strum(ascii_case_insensitive,to_string = "eth")]
    ETH,
    #[strum(ascii_case_insensitive,to_string = "usdt")]
    USDT,
    #[strum(ascii_case_insensitive,to_string = "usdc")]
    USDC,
    #[strum(ascii_case_insensitive,to_string = "cly")]
    CLY,
    #[strum(ascii_case_insensitive,to_string = "dw20")]
    DW20,
}

impl CoinType {
    pub fn to_account_id(&self) -> AccountId {
        match self {
            CoinType::BTC => AccountId::from_str("btc").unwrap(),
            CoinType::ETH => AccountId::from_str("eth").unwrap(),
            CoinType::USDT => AccountId::from_str("usdt").unwrap(),
            CoinType::USDC => AccountId::from_str("usdc").unwrap(),
            CoinType::CLY => AccountId::from_str("cly").unwrap(),
            CoinType::DW20 => AccountId::from_str("dw20").unwrap(),
        }
    }

    pub fn erc20_ca(&self) -> Option<String> {
        match self {
            CoinType::BTC => Some(global_conf.eth_wbtc_contract.clone()),
            CoinType::ETH => None,
            CoinType::USDT => Some(global_conf.eth_usdt_contract.clone()),
            CoinType::USDC => Some(global_conf.eth_usdc_contract.clone()),
            CoinType::CLY => None,
            CoinType::DW20 => Some(global_conf.eth_dw20_contract.clone()),
        }
    }

    pub fn to_account_str(&self) -> String {
        self.to_account_id().to_string()
    }
}

/****
impl AddressConvert for AccountId {
    fn to_account_str(&self) -> String {
        self.to_string()
    }
    fn from_account_str(s: &str) -> Result<Self, String> {
        AccountId::from_str(s).map_err(|x| x.to_string())
    }
}
*/

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone, EnumString, Display,Eq)]
pub enum CoinTxStatus {
    Created,
    SenderSigCompleted,
    //子账户是接收者需要特殊对待
    SenderSigCompletedAndReceiverIsSub,
    SenderSigCompletedAndReceiverIsBridge,
    ReceiverApproved,
    ReceiverRejected,
    SenderCanceled,
    SenderReconfirmed,
    MultiSigExpired,
    //上链后，由于合约复杂度没有立即finalize
    ChainPending,
    //如果commit之后没有finalize，更正Fail和Success的逻辑放在tx_list里面进行检查更新
    FinalizeAndFailed,
    FinalizeAndSuccessful,
}

/****
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
pub enum SecretKeyType {
    Master,
    Servant,
}

impl fmt::Display for SecretKeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretKeyType::Master => "Master",
            SecretKeyType::Servant => "Servant",
        };
        write!(f, "{}", description)
    }
}

impl FromStr for SecretKeyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Master" => Ok(SecretKeyType::Master),
            "Servant" => Ok(SecretKeyType::Servant),
            _ => Err("Don't support this".to_string()),
        }
    }
}
*/

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CoinTransaction {
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub status: CoinTxStatus,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<String>,
    pub tx_type: TxType,
    pub reserved_field1: String,
    pub reserved_field2: String,
    pub reserved_field3: String,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CoinTransaction2 {
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: String,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub status: CoinTxStatus,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<String>,
    pub tx_type: TxType,
    pub reserved_field1: String,
    pub reserved_field2: String,
    pub reserved_field3: String,
}


#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display,PartialEq)]
pub enum TxRole {
    #[strum(serialize = "Sender",to_string = "sender")]
    Sender,
    #[strum(serialize = "Receiver",to_string = "receiver")]
    Receiver
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display,PartialEq)]
pub enum TxType {
    Normal,
    Forced,
    MainToSub,
    SubToMain,
    MainToBridge,
}

impl TxRole {
    pub fn counterparty(&self) -> Self{
        match self {
            TxRole::Sender => TxRole::Receiver,
            TxRole::Receiver => TxRole::Sender,
        }
    }
}



#[derive(Deserialize,Serialize,PartialEq, Clone, Debug, Eq, Hash,EnumString,ToString)]
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
    NewcomerSwitchMaster
}


#[derive(Deserialize,Serialize,PartialEq, Clone, Debug, Eq, Hash,EnumString,ToString)]
pub enum TxStatusOnChain {
    Pending,
    FinalizeAndFailed,
    FinalizeAndSuccessful,
}

#[derive(Deserialize, Serialize,PartialEq, Clone, Debug, Eq, Hash)]
pub struct WalletManageRecord{
    pub record_id: String,
    pub user_id: String,
    pub operation_type: WalletOperateType,
    pub operator_pubkey: String,
    pub operator_device_id: String,
    pub operator_device_brand: String,
    pub tx_ids: Vec<String>,
    pub status: TxStatusOnChain,
}
