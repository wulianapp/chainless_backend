
use near_crypto::SecretKey;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality, FunctionArgs};
use std::ops::Deref;
use std::str::FromStr;
use tracing::debug;

use hex;
use lazy_static::lazy_static;
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::Action::FunctionCall;
use near_primitives::views::QueryRequest;

use common::data_structures::wallet::{CoinTransaction, CoinType};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::multi_sig::get_pubkey;
use crate::ContractClient;
use anyhow::{Ok, Result};


pub struct FeesCall {}
impl ContractClient<FeesCall> {
    //fixme: gen once object
    pub fn new() -> Result<Self> {
        let prikey_str = &common::env::CONF.multi_sig_relayer_prikey;
        //cvault0001.chainless
        let contract = &common::env::CONF.fees_call_contract;
        println!("___{}",prikey_str);
        let pri_key: SecretKey = prikey_str.parse()?;
        let pubkey = get_pubkey(&pri_key.to_string())?;

        let account_id = AccountId::from_str(&pubkey)?;

        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);
        Ok(Self {
            deployed_at: contract.parse()?,
            relayer: signer,
            phantom: Default::default(),
        })
    }

    pub async fn set_fees_priority(&self,account_id:&str,tokens:Vec<CoinType>) -> Result<String>{
        //todo: verify user's ecdsa signature
        let account_id = AccountId::from_str(&account_id)?;
        let tokens:Vec<AccountId> = tokens.iter().map(|coin|{
            coin.to_account_id()
        }).collect();
        let args_str = json!({
            "user_id":  account_id,
            "tokens": tokens,
        }).to_string();
        self.commit_by_relayer("set_user_tokens_admin", &args_str).await
    }


    pub async fn get_fees_priority(&self,account_id:&str) -> Result<Vec<CoinType>>{
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "id": user_account_id,
        }).to_string();
        //contract return default priority when not set
        let tokens:Option<Vec<String>> = self.query_call("get_user_tokens", &args_str).await?;
        let tokens = tokens
        .unwrap()
        .iter()
        .map(|x| x.parse::<CoinType>().map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<CoinType>>>()?;
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_fees_set_get() {
        let default_prioritys = vec![
            CoinType::USDC,
            CoinType::BTC,
            CoinType::ETH,
            CoinType::USDT,
            CoinType::DW20,
        ];
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let prioritys = fees_cli.get_fees_priority("user.node0").await.unwrap();
        println!("prioritys1 {:?} ",prioritys);
        let json_string = serde_json::to_string(&prioritys).unwrap();
        println!("prioritys_json1 {:?} ",json_string);


        let set_res = fees_cli.set_fees_priority("user.node0",default_prioritys).await.unwrap();
        println!("set_res {} ",set_res);


        let prioritys = fees_cli.get_fees_priority("user.node0").await.unwrap();
        println!("prioritys2 {:?} ",prioritys);
    }
}