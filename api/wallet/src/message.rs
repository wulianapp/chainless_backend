use std::collections::HashMap;
use std::ops::Deref;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};


lazy_static! {
    //user_id, message
    pub static ref MESSAGE_STORE: Mutex<HashMap<String, Vec<Message>>> = Mutex::new(HashMap::new());
    pub static ref MESSAGE_ID: Mutex<u128> = Mutex::new(0u128);
}
#[derive(Deserialize, Serialize,Clone)]
pub enum MessageType {
    //todo: should be key slice,but current it is all shards
    SyncPrivateKeyShard,
    //tx hex rawdata
    UnsignedTransaction,
    //result of mpc phase1
    UnsignedDataForPhase2,
    //some transactions require approval to receive
    PreNewIncome,
    //transaction have been confirmed on chain
    ConfirmedNewIncome,
    //user's send request is refused by receiver,data is tx hex rawdata
    RefusedSendMoney,
}

#[derive(Deserialize, Serialize,Clone)]
pub struct Message{
    pub id:u128,
    pub message_type:MessageType,
    pub data: String,
}