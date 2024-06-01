use std::str::FromStr;
use common::env::RelayerPool;
use lazy_static::lazy_static;


use common::data_structures::TxStatusOnChain;
use common::utils::math::hex_to_bs58;
use near_crypto::{InMemorySigner, KeyType, PublicKey, SecretKey, Signature};
use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitResponse;
use near_jsonrpc_client::methods::EXPERIMENTAL_check_tx::SignedTransaction;
use near_jsonrpc_client::{methods, MethodCallResult};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::views::{AccessKeyList, ExecutionStatusView, FinalExecutionStatus};
use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::Transaction;
use near_primitives::types::{AccountId, BlockReference};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;

use hex;
//use log::debug;
use anyhow::{anyhow, Result};
use tokio::sync::{Mutex, MutexGuard};
use tracing::{debug, error, warn};

pub struct Relayer {
    pub derive_index: u16,
    pub signer: InMemorySigner,
    //pub nonce: u64,
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

pub async fn wait_for_idle_relayer() -> MutexGuard<'static, Relayer> {
    loop {
        match find_idle_relayer() {
            Some(x) => {
                debug!("find idle relayer {}",x.derive_index);
                return x;
            }
            None => {
                warn!("relayer is busy");
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                continue;
            }
        }
    }
}

//自定义派生规则,28个字符基础seed，拼接index
pub fn chainless_sub_signer(account_id:&str, seed: &str,index:u16) -> Result<InMemorySigner>{
    assert_eq!(seed.len(),28);
    let seed = format!("{}{}",seed,index);
    let secret_key = SecretKey::from_seed(KeyType::ED25519, &seed);
    let account_id = AccountId::from_str(account_id)?;
    let signer = near_crypto::InMemorySigner::from_secret_key(account_id, secret_key);
    Ok(signer)
}


#[cfg(test)]
mod tests {
    use near_crypto::Signer;
    use near_primitives::{account::{AccessKey, AccessKeyPermission}, action::{Action, AddKeyAction}};
    use tracing::error;
    use common::log::init_logger;
    use crate::relayer::wait_for_idle_relayer;
    use super::*;

    #[tokio::test]
    async fn test_relayer_pool() {
        init_logger();
        let mut handles = vec![];
        for index in 0..10 {
            let handle = tokio::spawn(async move {
                //tokio::time::sleep(std::time::Duration::from_millis(index as u64 * 1000)).await;
                let relayer = wait_for_idle_relayer().await;
                error!("relayer {} index {}", relayer.signer.public_key.to_string(), index);
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
        let seed = "4eeFbz7F1ds17G1HBeUbJXPVYzzV";
        let account_id = AccountId::from_str("test").unwrap();
        let used_pubkey = PublicKey::from_str("ed25519:CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY").unwrap();
        let pri_key: SecretKey = "ed25519:3rSERwSqqyRNwSMaP61Kr3P96dQQGk4QwznTDNTxDMUqwTwkbBnjbwAjF39f98JSQzGXnzRWDUKb4HcpzDWyzWDc".parse().unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id.clone(), pri_key);
        //let account_id = "local";
        //let used_pubkey = "ed25519:9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP"
        let derive_size = 1;
        let mut actions = vec![];
        for index in 1..=100 {
            let key = chainless_sub_signer(&account_id.to_string(),&seed, index).unwrap();
            println!("key {}",key.public_key.to_string());
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
        .await.unwrap();    
        let current_nonce = match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => panic!(),
        };
        let tx = Transaction {
            signer_id: account_id.clone(),
            public_key: used_pubkey,
            nonce: current_nonce + 1,
            receiver_id: account_id,
            block_hash: access_key_query_response.block_hash,
            actions: actions,
        };

        let hash = tx.get_hash_and_size().0.as_bytes().to_owned();
        let _txid = hex::encode(hash);

        let signature = signer.sign(hash.as_slice());//sign(&hash);

        let tx = SignedTransaction::new(signature, tx);
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: tx.clone(),
        };
        let req = crate::rpc_call(request).await.unwrap();
        let txid2 = req.transaction.hash.to_string();
        println!("call commit_by_relayer txid {}", txid2);
    }
}
