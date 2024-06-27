//#![deny(warnings)]
//#![allow(unused_imports)]
#![allow(dead_code)]

pub mod airdrop;
pub mod bridge_on_near;
pub mod coin;
pub mod erc20_on_eth;
pub mod general;
pub mod multi_sig;

pub mod bridge_on_eth;
pub mod eth_cli;
pub mod fees_call;
mod relayer;

use general::pubkey_from_hex_str;
use lazy_static::lazy_static;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use anyhow::{anyhow, Result};
use common::prelude::*;
use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    borsh::{self},
    transaction::{
        Action, AddKeyAction, CreateAccountAction, DeleteKeyAction, FunctionCallAction,
        SignedTransaction, Transaction, TransferAction,
    },
    types::{AccountId, BlockReference, Finality, FunctionArgs},
    views::{FinalExecutionStatus, QueryRequest},
};
use relayer::{wait_for_idle_relayer, Relayer};
use serde::de::DeserializeOwned;

use std::marker::PhantomData;
use tokio::sync::MutexGuard;
use tracing::{debug, info};

use crate::general::gen_transaction_with_caller_with_nonce;

lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = {
        println!("+++__{}",common::env::CONF.chain_rpc);
        JsonRpcClient::connect(&common::env::CONF.chain_rpc)
    };



    //static ref CODE_STORAGE: Mutex<HashMap<(String, Usage), Captcha>> = Mutex::new(HashMap::new());

}

//todo: deal with error detail
pub async fn rpc_call<M>(
    method: M,
) -> Result<M::Response, near_jsonrpc_client::errors::JsonRpcError<M::Error>>
where
    M: methods::RpcMethod,
{
    for index in 0..5 {
        match crate::CHAIN_CLIENT.call(&method).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if index == 4 {
                    return Err(err);
                }
            }
        }
    }
    //Err(anyhow::anyhow!("call rpc failed, and max retries exceeded"))
    unreachable!()
}

pub struct ContractClient<T> {
    pub deployed_at: AccountId,
    //对于query的访问不需要签名，给none
    pub relayer: Option<MutexGuard<'static, Relayer>>,
    phantom: PhantomData<T>,
}

impl<T> AsRef<InMemorySigner> for ContractClient<T> {
    fn as_ref(&self) -> &InMemorySigner {
        //query的禁用
        &self.relayer.as_ref().unwrap().signer
    }
}

impl<T> Drop for ContractClient<T> {
    fn drop(&mut self) {
        if let Some(relayer) = self.relayer.as_ref() {
            debug!("index_relayer_{} released", relayer.derive_index);
        }
    }
}

impl<T> ContractClient<T> {
    //对于复用代码的项目需要手动注入地址
    pub async fn gen_cli(contract: &str) -> Result<Self> {
        let relayer = wait_for_idle_relayer().await?;
        Ok(Self {
            deployed_at: contract.parse()?,
            relayer: Some(relayer),
            phantom: Default::default(),
        })
    }

    pub async fn gen_cli_without_relayer(contract: &str) -> Result<Self> {
        Ok(Self {
            deployed_at: contract.parse()?,
            relayer: None,
            phantom: Default::default(),
        })
    }

    async fn gen_tx(
        &self,
        caller_account_id: &AccountId,
        caller_pubkey: &PublicKey,
        method_name: &str,
        args: &str,
    ) -> Result<Transaction> {
        //todo: when mainnet deposit is zero，now is 100 * cost
        let (receiver_str, actions, nonce) = if method_name == "register_account" {
            let args: Vec<&str> = args.split(':').collect();
            let account_id = args[0];
            let pubkey = args[1];

            let add_action = Action::AddKey(Box::new(AddKeyAction {
                public_key: pubkey_from_hex_str(pubkey)?,
                access_key: AccessKey {
                    nonce: 0u64,
                    permission: AccessKeyPermission::FullAccess,
                },
            }));
            let create_action = Action::CreateAccount(CreateAccountAction {});
            (account_id.to_string(), vec![create_action, add_action], 1)
        } else if method_name == "add_key" {
            let add_action = Action::AddKey(Box::new(AddKeyAction {
                public_key: pubkey_from_hex_str(args)?,
                access_key: AccessKey {
                    nonce: 0u64,
                    permission: AccessKeyPermission::FullAccess,
                },
            }));
            (caller_account_id.to_string(), vec![add_action], 1)
        } else if method_name == "delete_key" {
            let delete_action = Action::DeleteKey(Box::new(DeleteKeyAction {
                public_key: pubkey_from_hex_str(args)?,
            }));
            (caller_account_id.to_string(), vec![delete_action], 2)
        } else {
            let call_action = Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: method_name.to_string(),
                args: args.as_bytes().to_vec(),
                gas: CHAINLESS_DEFAULT_GAS_LIMIT, // 100 TeraGas
                deposit: None,
            }));
            (self.deployed_at.to_string(), vec![call_action], 1)
        };

        debug!("{}---{:?}", receiver_str, actions);
        let mut transaction = gen_transaction_with_caller_with_nonce(
            caller_account_id.to_owned(),
            caller_pubkey.to_owned(),
            &receiver_str,
            nonce,
        )
        .await?;
        transaction.actions = actions;
        Ok(transaction)
    }

    async fn gen_raw_with_caller(
        &self,
        caller_account_id: &AccountId,
        caller_pubkey: &PublicKey,
        method_name: &str,
        args: &str,
    ) -> Result<(String, String)> {
        let tx = self
            .gen_tx(caller_account_id, caller_pubkey, method_name, args)
            .await?;

        let raw_bytes = borsh::to_vec(&tx.clone())?;
        let raw_str = hex::encode(raw_bytes);

        let hash = tx.get_hash_and_size().0.as_bytes().to_owned();
        let txid = hex::encode(hash);

        Ok((txid, raw_str))
    }

    async fn commit_by_relayer(&mut self, method_name: &str, args: &str) -> Result<String> {
        debug!("method_name: {},args: {}", method_name, args);
        let mut transaction = self
            .gen_tx(
                &self.as_ref().account_id,
                &self.as_ref().public_key,
                method_name,
                args,
            )
            .await?;
        //todo: relayer用到的不止这一个地方
        let this_nonce = self.relayer.as_ref().unwrap().nonce.ok_or(anyhow!(""))? + 1;
        info!("index_relayer_{}_this_nonce {}", self.relayer.as_ref().unwrap().derive_index,this_nonce);
        self.relayer.as_mut().unwrap().nonce = Some(this_nonce);
        transaction.nonce = this_nonce;
        //relayer_sign
        let signature = self
            .as_ref()
            .sign(transaction.get_hash_and_size().0.as_ref());

        let tx = SignedTransaction::new(signature, transaction.clone());
        //todo: commit是否有必要，直接用async？
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };

        debug!("call commit_by_relayer txid {}", &tx.get_hash().to_string());

        let rep = crate::rpc_call(request).await?;
        if let FinalExecutionStatus::Failure(error) = rep.status {
            Err(anyhow!(error.to_string()))?
        }
        let _txid = rep.transaction.hash.to_string();

        let hash = transaction.get_hash_and_size().0.as_bytes().to_owned();
        let txid = hex::encode(hash);
        debug!("call commit_by_relayer2 txid {}", txid);
        Ok(txid)
    }

    pub async fn clear_all(&mut self) -> Result<String> {
        self.commit_by_relayer("clear_all", "").await
    }

    async fn query_call<R: DeserializeOwned>(
        &self,
        method_name: &str,
        args: &str,
    ) -> Result<Option<R>> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: (self.deployed_at).clone(),
                method_name: method_name.to_string(),
                args: FunctionArgs::from(args.to_string().into_bytes()),
            },
        };
        let rep = crate::rpc_call(request).await?;

        if let QueryResponseKind::CallResult(result) = rep.kind {
            let amount_str: String = String::from_utf8(result.result)?;
            debug!("query_res1 {}", amount_str);
            println!("query_res1 {}", amount_str);
            Ok(serde_json::from_str::<Option<R>>(&amount_str)?)
        } else {
            Err(anyhow!("kind must be contract call".to_string()))?
        }
    }
}