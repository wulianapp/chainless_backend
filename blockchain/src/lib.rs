#![allow(unused_imports)]
#![allow(dead_code)]

mod airdrop;
pub mod bridge_on_near;
pub mod coin;
pub mod erc20_on_eth;
pub mod general;
mod hello;
pub mod multi_sig;
mod newbie_reward;

pub mod bridge_on_eth;
pub mod eth_cli;
pub mod fees_call;

use ethers::providers::JsonRpcError;
use general::{gen_transaction_with_caller, pubkey_from_hex_str};
use lazy_static::lazy_static;
use near_jsonrpc_client::{methods, JsonRpcClient, MethodCallResult};
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use anyhow::{anyhow, Result};
use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    borsh::{self, BorshSerialize},
    transaction::{
        Action, AddKeyAction, CreateAccountAction, DeleteKeyAction, FunctionCallAction,
        SignedTransaction, Transaction, TransferAction,
    },
    types::{AccountId, BlockReference, Finality, FunctionArgs},
    views::{FinalExecutionStatus, QueryRequest},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Pointer, hash::Hash, marker::PhantomData, str::FromStr};
use tracing::{debug, error, field::debug, info};

use crate::general::{gen_transaction, gen_transaction_with_caller_with_nonce};

lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = {
        println!("+++__{}",common::env::CONF.chain_rpc);
        JsonRpcClient::connect(&common::env::CONF.chain_rpc)
    };
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

#[derive(Clone)]
pub struct ContractClient<T> {
    pub deployed_at: AccountId,
    relayer: InMemorySigner,
    phantom: PhantomData<T>,
}

impl<T> ContractClient<T> {
    async fn gen_tx(
        &self,
        caller_account_id: &AccountId,
        caller_pubkey: &PublicKey,
        method_name: &str,
        args: &str,
    ) -> Result<Transaction> {
        //todo: when mainnet deposit is zero，now is 100 * cost
        let (receiver_str, actions, nonce) = if method_name == "register_account" {
            (
                args.to_string(),
                //匿名账户可以通过转账的方式创建
                vec![Action::Transfer(TransferAction { deposit: 1u128 })],
                1,
            )
        //根据指定名字创建用户，需要配置顶级账户
        } else if method_name == "register_account_with_name" {
            let args: Vec<&str> = args.split(":").collect();
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
                gas: 600_000_000_000_000, // 100 TeraGas
                deposit: 0,
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

    /***
    async fn gen_create_account_tx(&self, receiver: &AccountId) -> Result<Transaction> {
        let deposit_actions: Vec<Action> =
            vec![Action::Transfer(TransferAction { deposit: 0u128 })];

        let mut transaction = gen_transaction_with_caller(
            self.relayer.account_id.clone(),
            self.relayer.public_key().clone(),
            receiver.as_str(),
        )
        .await?;
        transaction.actions = deposit_actions;
        Ok(transaction)
    }
    */
    async fn gen_raw_with_relayer(
        &self,
        method_name: &str,
        args: &str,
    ) -> Result<(String, String)> {
        self.gen_raw_with_caller(
            &self.relayer.account_id,
            &self.relayer.public_key(),
            method_name,
            args,
        )
        .await
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

    async fn commit_by_relayer(&self, method_name: &str, args: &str) -> Result<String> {
        debug!("method_name: {},args: {}",method_name,args);
        let transaction = self
            .gen_tx(
                &self.relayer.account_id,
                &self.relayer.public_key(),
                method_name,
                args,
            )
            .await?;
        //relayer_sign
        let signature = self
            .relayer
            .sign(transaction.get_hash_and_size().0.as_ref());

        let tx = SignedTransaction::new(signature, transaction.clone());
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };

        debug!("call commit_by_relayer txid {}", &tx.get_hash().to_string());

        let rep = crate::rpc_call(request).await.unwrap();
        if let FinalExecutionStatus::Failure(error) = rep.status {
            Err(anyhow!(error.to_string()))?
        }
        let txid = rep.transaction.hash.to_string();

        let hash = transaction.get_hash_and_size().0.as_bytes().to_owned();
        let txid = hex::encode(hash);
        debug!("call commit_by_relayer2 txid {}", txid);
        Ok(txid)
    }

    pub async fn clear_all(&self) -> Result<String> {
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
            Err(anyhow!(
                "kind must be contract call".to_string()
            ))?
        }
    }
}

pub async fn test_connect() {
    let mainnet_client = JsonRpcClient::connect("http://120.232.251.101:8061");
    let tx_status_request = methods::tx::RpcTransactionStatusRequest {
        transaction_info: TransactionInfo::TransactionId {
            tx_hash: "2HvMg8EpsgweGFSG87ngpJ97yWnuX9nBNB9yaXn8HC8w"
                .parse()
                .unwrap(),
            sender_account_id: "node0".parse().unwrap(),
        },
    };

    // call a method on the server via the connected client
    let tx_status = mainnet_client.call(tx_status_request).await.unwrap();

    println!("{:?}", tx_status);
}
/***
todo:
1、airdrop interface
2、newbie awards
3、mpc combine
4、transaction broadcast
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_sdk() {
        test_connect().await;
    }
}
