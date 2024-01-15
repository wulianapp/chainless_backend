use std::fmt;
use std::str::FromStr;

use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};

const PREDECESSOR_SUBFIX: &'static str = ".node0";

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

#[derive(Deserialize, Serialize, Debug)]
pub enum CoinType {
    CLY,
    DW20,
}

impl AddressConvert for CoinType {
    fn to_account_str(&self) -> String {
        match self {
            CoinType::CLY => "cly".to_string() + PREDECESSOR_SUBFIX,
            CoinType::DW20 => "dw20".to_string() + PREDECESSOR_SUBFIX,
        }
    }
    fn from_account_str(s: &str) -> Result<Self, String> {
        let id_str = s.replace(PREDECESSOR_SUBFIX, "");
        match id_str.as_str() {
            "cly" => Ok(CoinType::CLY),
            "dw20" => Ok(CoinType::DW20),
            _ => Err("Don't support this coin".to_string()),
        }
    }
}

impl AddressConvert for AccountId {
    fn to_account_str(&self) -> String {
        self.to_string()
    }
    fn from_account_str(s: &str) -> Result<Self, String> {
        AccountId::from_str(s).map_err(|x| x.to_string())
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
//#[serde(rename_all = "lowercase")]
pub enum CoinTxStatus {
    Created,
    ReceiverApproved,
    ReceiverRejected,
    SenderCanceled,
    SenderReconfirmed,
    Broadcast,
    Confirmed,
}

impl fmt::Display for CoinTxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxStatus::Created => "Created",
            CoinTxStatus::ReceiverApproved => "ReceiverApproved",
            CoinTxStatus::ReceiverRejected => "ReceiverRejected",
            CoinTxStatus::SenderCanceled => "SenderCanceled",
            CoinTxStatus::SenderReconfirmed => "SenderReconfirmed",
            CoinTxStatus::Broadcast => "Broadcast",
            CoinTxStatus::Confirmed => "Confirmed",
        };
        write!(f, "{}", description)
    }
}

impl FromStr for CoinTxStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Created" => Ok(CoinTxStatus::Created),
            "ReceiverApproved" => Ok(CoinTxStatus::ReceiverApproved),
            "ReceiverRejected" => Ok(CoinTxStatus::ReceiverRejected),
            "SenderCanceled" => Ok(CoinTxStatus::SenderCanceled),
            "SenderReconfirmed" => Ok(CoinTxStatus::SenderReconfirmed),
            "Broadcast" => Ok(CoinTxStatus::Broadcast),
            "Confirmed" => Ok(CoinTxStatus::Confirmed),
            _ => Err("Don't support this service mode".to_string()),
        }
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTransaction {
    pub tx_id: String,
    pub coin_type: CoinType,
    pub sender: u32,   //uid
    pub receiver: u32, //uid
    pub amount: u128,
    pub status: CoinTxStatus,
    pub raw_data: String,
    pub signatures: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Wallet {
    pub user_id: u32,
    pub account_id: String,
    pub sub_pubkeys: Vec<String>,
    pub sign_strategies: Vec<String>,
    pub participate_device_ids: Vec<String>,
}
