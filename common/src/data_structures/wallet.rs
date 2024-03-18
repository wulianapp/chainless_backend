use std::fmt;
use std::str::FromStr;

use super::secret_store::SecretStore;
use near_primitives::types::AccountId;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, ToString};

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
    CoinTx(u32, CoinTransaction),
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
        CoinType::BTC,
        CoinType::ETH,
        CoinType::USDT,
        CoinType::USDC,
        CoinType::DW20,
    ]
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display,PartialEq)]
pub enum CoinType {
    BTC,
    ETH,
    USDT,
    USDC,
    CLY,
    DW20,
}

impl CoinType {
    pub fn to_account_id(&self) -> AccountId {
        match self {
            CoinType::BTC => AccountId::from_str("btc.node0").unwrap(),
            CoinType::ETH => AccountId::from_str("eth.node0").unwrap(),
            CoinType::USDT => AccountId::from_str("usdt.node0").unwrap(),
            CoinType::USDC => AccountId::from_str("usdc.node0").unwrap(),
            CoinType::CLY => AccountId::from_str("cly.node0").unwrap(),
            CoinType::DW20 => AccountId::from_str("dw20.node0").unwrap(),
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
    ReceiverApproved,
    ReceiverRejected,
    SenderCanceled,
    SenderReconfirmed,
    Expired,
    Broadcast,
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
}
