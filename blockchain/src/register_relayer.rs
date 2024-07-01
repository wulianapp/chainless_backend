use common::env::RelayerPool;

use lazy_static::lazy_static;
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use std::cell::RefCell;
use std::str::FromStr;

use near_crypto::{InMemorySigner, KeyType, SecretKey};

use near_primitives::{hash::hash, types::BlockReference};

use near_primitives::types::AccountId;

//use log::debug;
use anyhow::{anyhow, Result};
use tokio::sync::{Mutex, MutexGuard};
use tracing::{debug, warn};
use crate::general::get_chain_state;

#[derive(Debug, Clone)]
pub struct RegisterRelayer {
    pub derive_index: u32,
    pub prikey: String,
    /// 用户注册的时候使用的max_block_height
    pub busy_height: Option<u64>,
}

lazy_static! {
    pub static ref REGISTER_RELAYER_POOL: Mutex<Vec<RegisterRelayer>> = {
        let RelayerPool { seed, account_id, derive_size }
            = common::env::CONF.relayer_pool.clone();
        let mut pool = vec![];
        for derive_index in 0..derive_size {
            let prikey = register_access_prikey(&seed,derive_index).unwrap();
            pool.push(RegisterRelayer{
                derive_index,
                prikey,
                busy_height: None
            });
        }
        Mutex::new(pool)
    };

}

pub async fn wait_for_idle_register_relayer() -> Result<RegisterRelayer> {
    let current_height = get_chain_state().await?.latest_block_height;
    loop {
        for relayer in REGISTER_RELAYER_POOL.lock().await.iter_mut() {
            if matches!(relayer.busy_height,Some(height) if height > current_height)  {
                continue;
            }else {
                relayer.busy_height = Some(current_height + 60);
                return Ok(relayer.to_owned())
            }
        }
        warn!("register relayer is busy");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}

//自定义派生规则,32字节随机值，拼接2字节index，
pub fn register_access_prikey(seed: &str, index: u32) -> Result<String> {
    if seed.len() < 64 {
        return Err(anyhow!("must be more than 64"));
    }
    let prikey_hex = format!("{}{:04x}", seed, index);
    let hash = hash(prikey_hex.as_bytes()).to_string();
    let secret_key = SecretKey::from_seed(KeyType::ED25519, &hash);
    Ok(secret_key.to_string())
}
