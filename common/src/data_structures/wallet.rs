use std::fmt;
use std::str::FromStr;
use crate::env::ServiceMode;
use near_primitives::types::{AccountId, Finality, FunctionArgs};

const PREDECESSOR_SUBFIX: &'static str = ".node0";

pub trait AddressConvert: Sized {
    fn to_user_id(&self) -> String {
        self.to_account_str().replace(PREDECESSOR_SUBFIX,"")
    }
    fn to_account_str(&self) -> String{
        self.to_user_id() + PREDECESSOR_SUBFIX
    }

    fn from_user_id(s:&str) -> Result<Self,String>{
        let new_s = s.to_owned() + PREDECESSOR_SUBFIX;
        Self::from_account_str(&new_s)
    }

    fn from_account_str( s:&str) -> Result<Self,String>
    {
        let new_s = s.replace(PREDECESSOR_SUBFIX,"");
        Self::from_user_id(&new_s)
    }

}

#[derive(Debug)]
pub enum CoinType{
    CLY,
    DW20,
}

impl AddressConvert for CoinType {
    fn to_user_id(&self) -> String {
        match self {
            CoinType::CLY => "cly".to_string(),
            CoinType::DW20 => "dw20".to_string(),
        }
    }
    fn from_user_id(s: &str) -> Result<Self, String> {
        match s {
            "cly" => Ok(CoinType::CLY),
            "dw20" => Ok(CoinType::DW20),
            _ => Err("Don't support this coin".to_string()),
        }
    }
}

impl AddressConvert for AccountId{
    fn to_account_str(&self) -> String {
        self.to_string()
    }
    fn from_account_str(s: &str) -> Result<Self, String> {
        AccountId::from_str(s).map_err(|x| x.to_string())
    }

}


#[derive(Debug)]
pub enum TransferStatus{
    Pending,
    Refused,
    //if this transaction is launched,update it to be confirmed immediately
    Confirmed,
}
#[derive(Debug)]
pub struct CoinTransfer {
    pub tx_id: String,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to:String,    //uid
    pub amount: u128,
    pub status: TransferStatus,
    pub created_at: u64,
    pub confirmed_at:u64,
}