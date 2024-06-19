use common::env::RelayerPool;

use lazy_static::lazy_static;
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use std::str::FromStr;

use near_crypto::{InMemorySigner, KeyType, SecretKey};

use near_primitives::{hash::hash, types::BlockReference};

use near_primitives::types::AccountId;

//use log::debug;
use anyhow::{anyhow, Result};
use tokio::sync::{Mutex, MutexGuard};
use tracing::{debug, warn};

pub struct Relayer {
    pub derive_index: u32,
    pub signer: InMemorySigner,
    /// 用到的当前的时候再去初始化，初始化完成之后再本地维护
    pub nonce: Option<u64>,
}

impl AsRef<InMemorySigner> for Relayer {
    fn as_ref(&self) -> &InMemorySigner {
        &self.signer
    }
}

lazy_static! {
    //use relayer array to avoid nonce conflict
    pub static ref MULTI_SIG_RELAYER_POOL: Vec<Mutex<Relayer>> = {
        let RelayerPool { seed, account_id, derive_size }
            = common::env::CONF.relayer_pool.clone();
        let mut pool = vec![];
        for derive_index in 1..=derive_size {
            let signer = chainless_sub_signer(&account_id,&seed,derive_index).unwrap();
            pool.push(Mutex::new(Relayer{
                derive_index,
                signer,
                nonce: None
            }));
        }
        pool
    };

}

pub fn find_idle_relayer() -> Option<MutexGuard<'static, Relayer>> {
    for relayer in MULTI_SIG_RELAYER_POOL.iter() {
        match relayer.try_lock() {
            Ok(guard) => {
                return Some(guard);
            }
            Err(_) => continue,
        }
    }
    None
}

pub async fn wait_for_idle_relayer() -> Result<MutexGuard<'static, Relayer>> {
    loop {
        match find_idle_relayer() {
            Some(mut x) => {
                let current_nonce = if let Some(num) = x.nonce{
                    num
                }else{
                    let access_key_query_response = crate::rpc_call(RpcQueryRequest {
                        block_reference: BlockReference::latest(),
                        request: near_primitives::views::QueryRequest::ViewAccessKey {
                            account_id: x.signer.account_id.clone(),
                            public_key: x.signer.public_key.clone(),
                        },
                    })
                    .await?;

                    let current_nonce = match access_key_query_response.kind {
                        near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) => access_key.nonce,
                        _ => Err(anyhow::anyhow!("failed to extract current nonce"))?,
                    };
                    current_nonce            
                };
                x.nonce = Some(current_nonce + 1);
                debug!("find idle index_relayer_{}", x.derive_index);
                return Ok(x);
            }
            None => {
                warn!("relayer is busy");
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                continue;
            }
        }
    }
}

//自定义派生规则,32字节随机值，拼接2字节index，
pub fn chainless_sub_signer(account_id: &str, seed: &str, index: u32) -> Result<InMemorySigner> {
    if seed.len() < 64 {
        return Err(anyhow!("must be more than 64"));
    }
    let prikey_hex = format!("{}{:04x}", seed, index);
    let hash = hash(prikey_hex.as_bytes()).to_string();
    let secret_key = SecretKey::from_seed(KeyType::ED25519, &hash);
    let account_id = AccountId::from_str(account_id)?;
    let signer = near_crypto::InMemorySigner::from_secret_key(account_id, secret_key);
    Ok(signer)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::relayer::wait_for_idle_relayer;
    use common::log::init_logger;
    use near_crypto::Signer;
    use near_primitives::{
        account::{AccessKey, AccessKeyPermission},
        action::{Action, AddKeyAction},
        types::BlockReference,
    };
    use near_crypto::{PublicKey, SecretKey};
    use near_jsonrpc_client::methods::EXPERIMENTAL_check_tx::SignedTransaction;
    use near_jsonrpc_client::methods;
    use near_primitives::transaction::Transaction;
    use near_primitives::types::AccountId;
    use tracing::error;

    #[tokio::test]
    async fn test_relayer_pool() {
        init_logger();
        let mut handles = vec![];
        for index in 0..10 {
            let handle = tokio::spawn(async move {
                //tokio::time::sleep(std::time::Duration::from_millis(index as u64 * 1000)).await;
                let relayer = wait_for_idle_relayer().await.unwrap();
                error!(
                    "relayer {} index {}",
                    relayer.signer.public_key.to_string(),
                    index
                );
                //tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
                index
            });
            handles.push(handle);
        }
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        assert_eq!(results, (0..10).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn test_tool_add_many_pubkey() {
        //let account_id = AccountId::from_str("test").unwrap();
        //let pri_key: SecretKey = "ed25519:3rSERwSqqyRNwSMaP61Kr3P96dQQGk4QwznTDNTxDMUqwTwkbBnjbwAjF39f98JSQzGXnzRWDUKb4HcpzDWyzWDc".parse().unwrap();
        //let used_pubkey = PublicKey::from_str("ed25519:CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY").unwrap();
        let account_id = AccountId::from_str("local").unwrap();
        let pri_key: SecretKey = "ed25519:3cBasJZvQutzkCV5qv4rF7aikPH3EWMTQM6mGbJnXJ6i9zAcdsaun82WQgzQbxxKmEVTBvS2NJDBeKk23FFb43kd".parse().unwrap();
        let used_pubkey =
            PublicKey::from_str("ed25519:9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP").unwrap();

        let seed = "e48815443073117d29a8fab50c9f3feb80439c196d4d9314400e8e715e231849";
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id.clone(), pri_key);
        let mut actions = vec![];
        for index in 10001u32..=100000u32 {
            let key = chainless_sub_signer(account_id.as_ref(), seed, index).unwrap();
            println!("key {}", key.public_key);
            let add_action = Action::AddKey(Box::new(AddKeyAction {
                public_key: key.public_key,
                access_key: AccessKey {
                    nonce: 0u64,
                    permission: AccessKeyPermission::FullAccess,
                },
            }));
            actions.push(add_action);
        }

        let access_key_query_response = crate::rpc_call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: account_id.clone(),
                public_key: used_pubkey.clone(),
            },
        })
        .await
        .unwrap();
        let current_nonce = match access_key_query_response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => panic!(),
        };
        let tx = Transaction {
            signer_id: account_id.clone(),
            public_key: used_pubkey,
            nonce: current_nonce + 1,
            receiver_id: account_id,
            block_hash: access_key_query_response.block_hash,
            actions,
        };

        let hash = tx.get_hash_and_size().0.as_bytes().to_owned();
        let _txid = hex::encode(hash);

        let signature = signer.sign(hash.as_slice()); //sign(&hash);

        let tx = SignedTransaction::new(signature, tx);
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };
        let req = crate::rpc_call(request).await.unwrap();
        let txid2 = req.transaction.hash.to_string();
        println!("call commit_by_relayer txid {}", txid2);
    }
}
