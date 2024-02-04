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

#[derive(Deserialize, Serialize, Debug,Clone)]
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
    SenderSigCompleted,
    ReceiverApproved,
    ReceiverRejected,
    SenderCanceled,
    SenderReconfirmed,
    Expired,
    Broadcast,
    Confirmed,
}

impl fmt::Display for CoinTxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxStatus::Created => "Created",
            CoinTxStatus::SenderSigCompleted => "SenderSigCompleted",
            CoinTxStatus::ReceiverApproved => "ReceiverApproved",
            CoinTxStatus::ReceiverRejected => "ReceiverRejected",
            CoinTxStatus::SenderCanceled => "SenderCanceled",
            CoinTxStatus::SenderReconfirmed => "SenderReconfirmed",
            CoinTxStatus::Broadcast => "Broadcast",
            CoinTxStatus::Expired => "Expired",
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
            "SenderSigCompleted" => Ok(CoinTxStatus::SenderSigCompleted),
            "ReceiverApproved" => Ok(CoinTxStatus::ReceiverApproved),
            "ReceiverRejected" => Ok(CoinTxStatus::ReceiverRejected),
            "SenderCanceled" => Ok(CoinTxStatus::SenderCanceled),
            "SenderReconfirmed" => Ok(CoinTxStatus::SenderReconfirmed),
            "Expired" => Ok(CoinTxStatus::Expired),
            "Broadcast" => Ok(CoinTxStatus::Broadcast),
            "Confirmed" => Ok(CoinTxStatus::Confirmed),
            _ => Err("Don't support this service mode".to_string()),
        }
    }
}




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


#[derive(Deserialize, Serialize, Debug,Clone)]
pub struct CoinTransaction {
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub sender: String,   //uid
    pub receiver: String, //uid
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub status: CoinTxStatus,
    pub raw_data: Option<String>,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Wallet2 {
    pub account_id: String,
    pub master_device_id: String,
    pub master_pubkey: String,
    pub servant_device_ids: Vec<String>,
    pub servant_pubkeys: Vec<String>,
    pub sign_strategies: String,
}
