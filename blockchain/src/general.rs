use common::data_structures::wallet::TxStatusOnChain;
use common::utils::math::hex_to_bs58;
use near_crypto::{InMemorySigner, KeyType, PublicKey, Signature};
use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitResponse;
use near_jsonrpc_client::methods::EXPERIMENTAL_check_tx::SignedTransaction;
use near_jsonrpc_client::{methods, MethodCallResult};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::views::{AccessKeyList, ExecutionStatusView, FinalExecutionStatus};
use std::str::FromStr;

use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::Transaction;
use near_primitives::types::{AccountId, BlockReference};

use near_jsonrpc_client::{JsonRpcClient};
use near_jsonrpc_primitives::types::transactions::TransactionInfo;


use hex;
//use log::debug;
use tracing::{debug, warn};
use anyhow::{anyhow, Ok, Result};

//todo: contract_addr type change into AccountId
pub async fn gen_transaction(signer: &InMemorySigner, contract_addr: &str) -> Result<Transaction> {
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        }).await?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ =>    Err(anyhow::anyhow!("failed to extract current nonce"))?
    };

    Ok(
    Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_addr.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![],
    })
}

pub async fn gen_transaction_with_caller(
    caller_account_id: AccountId,
    caller_pubkey: PublicKey,
    contract_addr: &str,
) -> Result<Transaction> {
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: caller_account_id.clone(),
                public_key: caller_pubkey.clone(),
            },
        })
        .await?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ =>  Err(anyhow::anyhow!("failed to extract current nonce"))?,
    };
    Ok(
    Transaction {
        signer_id: caller_account_id,
        public_key: caller_pubkey,
        nonce: current_nonce + 1,
        receiver_id: contract_addr.parse().unwrap(),
        block_hash: access_key_query_response.block_hash,
        actions: vec![],
    })
}

pub async fn gen_transaction_with_caller_with_nonce(
    caller_account_id: AccountId,
    caller_pubkey: PublicKey,
    contract_addr: &str,
    add_nonce: u8,
) -> Result<Transaction> {
    debug!("{},{},{},{}",caller_account_id.to_string(),
    caller_pubkey.to_string(),
    contract_addr.to_string(),
    add_nonce.to_string()
);
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: caller_account_id.clone(),
                public_key: caller_pubkey.clone(),
            },
        })
        .await?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err(anyhow::anyhow!("failed to extract current nonce"))?,
    };
    Ok(Transaction {
        signer_id: caller_account_id,
        public_key: caller_pubkey,
        nonce: current_nonce + add_nonce as u64,
        receiver_id: contract_addr.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![],
    })
}

pub fn account_id_from_hex_str(id: &str) -> Result<AccountId> {
    let id_bytes = hex::decode(id)?;
    let account_id = AccountId::try_from_slice(&id_bytes)?;
    Ok(account_id)
}

pub fn pubkey_from_hex_str(key: &str) -> Result<PublicKey> {
    let account_id = AccountId::from_str(key)?;
    let key = PublicKey::from_near_implicit_account(&account_id)?;
    Ok(key)
}

pub async fn get_access_key_list(account_str: &str) -> Result<AccessKeyList> {
    let account_id = AccountId::from_str(account_str)?;
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKeyList { account_id },
        })
        .await?;

    match access_key_query_response.kind {
        QueryResponseKind::AccessKeyList(list) => Ok(list),
        _ =>  Err(anyhow::anyhow!("failed to extract current nonce"))?,
    }
}



pub async fn tx_status(tx_id: &str) -> Result<TxStatusOnChain> {
   
    let tx_status_request = methods::tx::RpcTransactionStatusRequest {
        transaction_info: TransactionInfo::TransactionId {
            tx_hash: tx_id.parse().unwrap(),
            sender_account_id: "node0".parse()?,
        },
    };

    let tx_status = crate::CHAIN_CLIENT.call(tx_status_request).await?;


    let status = if let FinalExecutionStatus::SuccessValue(_value) = tx_status.status {
        let mut status =   TxStatusOnChain::FinalizeAndSuccessful;
        for outcome in tx_status.receipts_outcome {
            match outcome.outcome.status{
                ExecutionStatusView::Unknown => unreachable!(""),
                ExecutionStatusView::Failure(_) => {
                    status = TxStatusOnChain::FinalizeAndFailed;
                },
                ExecutionStatusView::SuccessValue(_) => {},
                ExecutionStatusView::SuccessReceiptId(_) => {},
            }
        }
        status
    }else if let FinalExecutionStatus::Failure(error) = tx_status.status{
        warn!("tx_id({}) is failed: {}",tx_id,error.to_string());
        TxStatusOnChain::FinalizeAndFailed
    }else {
        TxStatusOnChain::Pending
    };
    Ok(status)
}

pub async fn safe_gen_transaction(
    caller_account_id: &str,
    caller_pubkey: &str,
    contract_addr: &str,
) -> Transaction {
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: AccountId::from_str(caller_account_id).unwrap(),
                public_key: PublicKey::from_str(caller_pubkey).unwrap(),
            },
        })
        .await
        .unwrap();

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => panic!("{:?}", "failed to extract current nonce"),
    };

    Transaction {
        signer_id: AccountId::from_str(caller_account_id).unwrap(),
        public_key: PublicKey::from_str(caller_pubkey).unwrap(),
        nonce: current_nonce + 1,
        receiver_id: contract_addr.parse().unwrap(),
        block_hash: access_key_query_response.block_hash,
        actions: vec![],
    }
}

//user-api shouldn't use this directly
pub async fn broadcast_tx_commit_from_raw(tx_str: &str, sig_str: &str) {
    let tx_hex = hex::decode(tx_str).unwrap();
    let sign_hex = hex::decode(sig_str).unwrap();
    let transaction = Transaction::deserialize(&mut tx_hex.as_slice()).unwrap();
    debug!("{:?}", transaction);
    let signature = Signature::from_parts(KeyType::ED25519, &sign_hex).unwrap();
    let rest = broadcast_tx_commit(transaction, signature).await;
    debug!("broadcast_tx_commit_from_raw {:?}", rest.status);
}

pub async fn broadcast_tx_commit_from_raw2(tx_str: &str, sig_str: &str) {
    let tx_hex = hex::decode(tx_str).unwrap();
    let sign_hex = hex::decode(sig_str).unwrap();
    let transaction = Transaction::deserialize(&mut tx_hex.as_slice()).unwrap();
    debug!("{:?}", transaction);
    //let signature = Signature::try_from_slice(&sign_hex).unwrap();
    let signature = Signature::from_parts(KeyType::ED25519, &sign_hex).unwrap();
    let rest = broadcast_tx_commit(transaction, signature).await;
    debug!("broadcast_tx_commit_from_raw {:?}", rest);
}

pub async fn broadcast_tx_commit(
    transaction: Transaction,
    sig_data: Signature,
) -> RpcBroadcastTxCommitResponse {
    let tx = SignedTransaction::new(sig_data, transaction);
    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: tx,
    };
    crate::CHAIN_CLIENT.call(request).await.unwrap()
}

pub async fn call<M>(request: M) -> MethodCallResult<M::Response, M::Error>
where
    M: methods::RpcMethod,
{
    crate::CHAIN_CLIENT.call(request).await
}
/***
pub async fn broadcast_tx_async(){
    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(&signer),
    };
    let tx_hash = crate::CHAIN_CLIENT.call(request).await.unwrap();
}

 */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_tx_commit_from_raw() {
        //generate form example.js,current is already Expired;
        let raw_sign = "11bfe4d0b7705f6c57282a9030b22505ce2641547e9f246561d75a284f5a6e0a10e596fa7e702b6f897ad19c859ee880d4d1e80e521d91c53cc8827b67558a0e";
        let raw_tx = "1d00000074696d657374616d705f313730343139303135343938332e6e6f64653000b07a2c1e6d6a5f42827bace780a4cd9b03d37b5cff85f2fdcd08821ecbc3db9181a2a6585a010000050000006e6f6465308fed8725a8d7494013680e18ee53e86c76598ff2734ca1739735e1b16fc9a829010000000301000000000000000000000000000000";
        let _res = broadcast_tx_commit_from_raw(raw_tx, raw_sign).await;
    }
}
