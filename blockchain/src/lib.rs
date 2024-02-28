#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(let_chains)]

mod airdrop;
pub mod coin;
pub mod general;
mod hello;
pub mod multi_sig;
mod newbie_reward;

use common::{error_code::{BackendError, ExternalServiceError}, http::{BackendRes, BackendRespond}};
use lazy_static::lazy_static;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use near_crypto::{InMemorySigner, Signer};
use near_primitives::{borsh::BorshSerialize, transaction::{Action, FunctionCallAction, SignedTransaction, Transaction}, types::{AccountId, BlockReference, Finality, FunctionArgs}, views::{FinalExecutionStatus, QueryRequest}};
use serde::{de::DeserializeOwned, Deserialize};
use tracing::{debug,info};
use std::marker::PhantomData;
use serde_json::json;
use common::error_code;

use crate::general::gen_transaction;


lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://120.232.251.101:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
}

#[derive(Clone)]
pub struct ContractClient<T> {
    pub deployed_at: AccountId,
    relayer: InMemorySigner,
    phantom: PhantomData<T>,
}

impl<T> ContractClient<T> {
    async fn gen_tx(&self,method_name:&str,args:&str) -> Transaction{
        let set_strategy_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
            method_name: method_name.to_string(),
            args: args.try_to_vec().unwrap(),
            gas: 300000000000000, // 100 TeraGas
            deposit: 0,
        }))];

        let mut transaction = gen_transaction(&self.relayer, &self.deployed_at.to_string()).await;
        transaction.actions = set_strategy_actions;
        transaction
    }
    async fn gen_raw(&self,method_name:&str,args:&str) -> BackendRes<(String,String)>{
        let tx = self.gen_tx(method_name,args).await;

        let hash = tx.get_hash_and_size().0.try_to_vec().unwrap();
        let txid = hex::encode(hash);

        let raw_bytes = tx.try_to_vec().unwrap();
        let raw_str = hex::encode(raw_bytes);
        Ok(Some((txid,raw_str)))
    }
    async fn commit_by_relayer(&self,method_name:&str,args:&str) -> BackendRes<String>{
        let transaction = self.gen_tx(method_name,args).await;
        //relayer_sign
        let signature = self
            .relayer
            .sign(transaction.get_hash_and_size().0.as_ref());

        let tx = SignedTransaction::new(signature, transaction);
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };

        debug!("call commit_by_relayer txid {}", &tx.get_hash().to_string());

        let rep = crate::general::call(request).await.unwrap();
        if let FinalExecutionStatus::Failure(error) = rep.status {
            Err(ExternalServiceError::Chain(error.to_string()))?
        }
        let txid = rep.transaction.hash.to_string();
        Ok(Some(txid))
    }

    pub async fn clear_all(&self) -> BackendRes<String>{
        self.commit_by_relayer("clear_all", "").await
    }


    async fn query_call<R:DeserializeOwned>(&self,method_name:&str,args:&str) -> BackendRes<R>{
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: (self.deployed_at).clone(),
                method_name: method_name.to_string(),
                args: FunctionArgs::from(
                    args.try_to_vec().unwrap(),
                ),
            },
        };
        let rep = crate::general::call(request).await.unwrap();

        if let QueryResponseKind::CallResult(result) = rep.kind {
            let amount_str: String = String::from_utf8(result.result).unwrap();
            if amount_str.eq("null"){
                Ok(None)
            }else {
                Ok(Some(serde_json::from_str::<R>(&amount_str).unwrap()))
            }
        } else {
            Err(BackendError::InternalError("kind must be contract call".to_string()))?
        }
    }
}

pub async fn test1() {
    let mainnet_client = JsonRpcClient::connect("http://120.232.251.101:8061");
    let tx_status_request = methods::tx::RpcTransactionStatusRequest {
        transaction_info: TransactionInfo::TransactionId {
            hash: "2HvMg8EpsgweGFSG87ngpJ97yWnuX9nBNB9yaXn8HC8w"
                .parse()
                .unwrap(),
            account_id: "node0".parse().unwrap(),
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
        test1().await;
    }
}
