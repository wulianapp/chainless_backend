/***
use std::collections::HashMap;
use std::ops::Deref;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use blockchain::coin::decode_coin_transfer;
use common::data_structures::wallet::{CoinTransaction, CoinType, TransferStatus};
use common::error_code::WalletError;
use common::utils::time::{get_unix_time, get_unix_time_nanos};
use crate::message;

//maybe as memory cache
lazy_static! {
    //message_id -> message_info, read is frequently and write is occasional
    pub static ref MESSAGE_STORE: Mutex<HashMap<u64, AppTransaction >> = Mutex::new(HashMap::new());
    //user_id -> message_id
    pub static ref USER_STORE: Mutex<HashMap<u32, Vec<u64>>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize, Serialize, Clone, Debug,PartialEq)]
pub enum MessageType {
    //todo: should be key slice,but current it is all shards
    MultiWalletRegister,
    Transaction,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeviceStatus {
    pub device_id: String,
    pub is_done: bool,
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppTransaction {
    pub tx_id: String,
    pub coin_type: CoinType,
    pub from: u32, //uid
    pub to:u32,    //uid
    pub amount: u128,
    pub status: AppTxStatus,
    pub created_at: u64,
}

impl AppTransaction {
    pub fn create(tx_raw: &str) -> Result<AppTransaction,Box<dyn std::error::Error>> {
        let CoinTransaction {tx_id,coin_type, sender: from, receiver: to,amount,created_at,..}
            = decode_coin_transfer(tx_raw)?;
        //let msg_id = get_unix_time_nanos();
        Ok(AppTransaction {
            tx_id,
            coin_type,
            from,
            to,
            amount,
            status: AppTxStatus::Created,
            created_at
        })
    }
}

pub fn get_all_message() -> Vec<(u64, AppTransaction)> {
    let store = message::MESSAGE_STORE.lock().unwrap();
    let store = store.iter()
        .map(|(uid, messages)| (*uid, messages.clone()))
        .collect::<Vec<(u64, AppTransaction)>>();
    store
}

//add filter status array
pub fn get_user_message(user_id: &u32) -> Option<Vec<AppTransaction>> {
    let msg_store = message::MESSAGE_STORE.lock().unwrap();
    let usr_store = message::USER_STORE.lock().unwrap();

    usr_store
        .get(user_id)
        .as_ref()
        .map(|x| {
            x.iter().map(|msg_id|
                //fixme:
                msg_store.get(msg_id).unwrap().to_owned()
            ).collect::<Vec<AppTransaction>>()
        })
}

pub fn insert_new_message(msg: AppTransaction) {
    let mut msg_store = message::MESSAGE_STORE.lock().unwrap();
    let mut usr_store = message::USER_STORE.lock().unwrap();

    msg_store.insert(msg.id, msg.clone());

    usr_store.entry(msg.from.clone())
        .or_insert_with(Vec::new)
        .push(msg.id);

    if let Some(id) = msg.to.clone() {
        usr_store.entry(id)
            .or_insert_with(Vec::new)
            .push(msg.id);
    }
}

pub fn update_message_status(message_id: u64, msg_status: MessageStatus) -> Result<(), WalletError> {
    //todo: check if message_id is belong user
    let mut msg_store = message::MESSAGE_STORE.lock().unwrap();
    if let Some(x) = msg_store.get_mut(&message_id) {
        x.status = msg_status;
        Ok(())
    } else {
        Err(WalletError::Unknown(format!("message_id {} not find", message_id)))
    }
}

pub fn delete_message(message_id: u64) -> Result<(), WalletError> {
    let mut msg_store = message::MESSAGE_STORE.lock().unwrap();
    let mut usr_store = message::USER_STORE.lock().unwrap();

    //todo: check if message_id is belong user
    //if message_id is nonexistent,should throw error
    if let Some(msg) = msg_store.remove(&message_id){
        if let Some(ids) = usr_store.get_mut(&msg.from){
            ids.retain(|x| *x != message_id);
        }else {
            return Err(WalletError::Unknown(format!("user_id {} not find", msg.from)));
        }

        if let Some(to) = msg.to {
            if let Some(ids) = usr_store.get_mut(&to){
                ids.retain(|x| *x != message_id);
            }else {
                return Err(WalletError::Unknown(format!("user_id {} not find", to)));
            }
        }
    }else {
        return Err(WalletError::Unknown(format!("message_id {}not find", message_id)));
    }
    Ok(())
}

 */
