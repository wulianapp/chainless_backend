use common::utils::math::coin_amount::raw2display;
use common::utils::math::hex_to_bs58;
use near_crypto::SecretKey;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality, FunctionArgs};
use std::ops::{Deref, Div};
use std::str::FromStr;
use tracing::debug;

use hex;
use lazy_static::lazy_static;
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::Action::FunctionCall;
use near_primitives::views::QueryRequest;

use common::data_structures::CoinType;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::multi_sig::get_pubkey;
use crate::ContractClient;
use anyhow::{Ok, Result};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct U128(pub u128);

pub struct FeesCall {}
impl ContractClient<FeesCall> {
    //fixme: gen once object
    pub fn new() -> Result<Self> {
        let prikey_str = &common::env::CONF.multi_sig_relayer_prikey;
        let relayer_account = &common::env::CONF.multi_sig_relayer_account_id;
        //cvault0001.chainless
        let contract = &common::env::CONF.fees_call_contract;
        println!("___{}", prikey_str);
        let pri_key: SecretKey = prikey_str.parse()?;
        let _pubkey = get_pubkey(&pri_key.to_string())?;

        let account_id = AccountId::from_str(relayer_account)?;

        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);
        Ok(Self {
            deployed_at: contract.parse()?,
            relayer: signer,
            phantom: Default::default(),
        })
    }

    pub async fn set_fees_priority(
        &self,
        account_id: &str,
        tokens: Vec<CoinType>,
    ) -> Result<String> {
        //todo: verify user's ecdsa signature
        let account_id = AccountId::from_str(&account_id)?;
        let tokens: Vec<AccountId> = tokens.iter().map(|coin| coin.to_account_id()).collect();
        let args_str = json!({
            "user_id":  account_id,
            "tokens": tokens,
        })
        .to_string();
        self.commit_by_relayer("set_user_tokens_admin", &args_str)
            .await
    }

    pub async fn get_fees_priority(&self, account_id: &str) -> Result<Vec<CoinType>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({
            "id": user_account_id,
        })
        .to_string();
        //contract return default priority when not set
        let tokens: Option<Vec<String>> = self.query_call("get_user_tokens", &args_str).await?;
        let tokens = tokens
            .unwrap()
            .iter()
            .map(|x| x.parse::<CoinType>().map_err(|e| anyhow::anyhow!(e)))
            .collect::<Result<Vec<CoinType>>>()?;
        Ok(tokens)
    }

    //后台不做乘法计算，允许这里精度丢失
    pub async fn get_coin_price(&self, coin: &CoinType) -> Result<(u128, u128)> {
        let args_str = json!({
            "id":  coin.to_account_id(),
        })
        .to_string();
        let (base_amount, quote_amount): (String, String) =
            self.query_call("get_price", &args_str).await?.unwrap();
        let base_amount: u128 = base_amount.parse()?;
        let quote_amount: u128 = quote_amount.parse()?;
        Ok((base_amount, quote_amount))
    }

    pub async fn get_coin_price_custom(&self, coin: &CoinType) -> Result<f32> {
        let (base_amount, quote_amount) = self.get_coin_price(coin).await?;
        let price = quote_amount as f32 / base_amount as f32;
        Ok(price)
    }

    //base_fee
    pub async fn get_tx_base_fee(&self, tx_id: &str) -> Result<(CoinType, u128)> {
        //let value = (user_id, fees_id, fees_amount, tx_hash, memo);
        //AccountId, AccountId, u128, Option<String>, String

        let args_str = json!({
            "hsh":  hex_to_bs58(tx_id)?,
        })
        .to_string();
        let (_user_id, fees_id, fees_amount, _tx_hash, _memo): (
            String,
            String,
            u128,
            Option<String>,
            String,
        ) = self
            .query_call("get_tx_with_hash", &args_str)
            .await?
            .unwrap();
        let coin: CoinType = fees_id.parse()?;
        Ok((coin, fees_amount))
    }

    pub async fn get_user_txs(
        &self,
        account_id: &str,
    ) -> Result<Vec<(String, u128, Option<String>, String)>> {
        //let value = (user_id, fees_id, fees_amount, tx_hash, memo);
        //AccountId, AccountId, u128, Option<String>, String

        let args_str = json!({
            "id":  AccountId::from_str(account_id)?,
        })
        .to_string();
        //        let (fees_id, fees_amount, tx_hash, _memo): Vec<(
        let all_tx: Vec<(String, u128, Option<String>, String)> = self
            .query_call("get_user_txs", &args_str)
            .await?
            .unwrap_or(vec![]);
        Ok(all_tx)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_fees_set_get() {
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let prioritys = fees_cli.get_fees_priority("user.node0").await.unwrap();
        println!("prioritys_1 {:?} ", prioritys);

        let default_prioritys = vec![
            CoinType::USDC,
            CoinType::BTC,
            CoinType::ETH,
            CoinType::USDT,
            CoinType::DW20,
        ];

        let json_string = serde_json::to_string(&prioritys).unwrap();
        println!("prioritys_json1 {:?} ", json_string);

        let set_res = fees_cli
            .set_fees_priority("user.node0", default_prioritys)
            .await
            .unwrap();
        println!("set_res {} ", set_res);

        let fee_info = fees_cli.get_tx_base_fee(&set_res).await.unwrap();
        println!("fee_info {:?} ", fee_info);

        let prioritys = fees_cli.get_fees_priority("user.node0").await.unwrap();
        println!("prioritys2 {:?} ", prioritys);
    }

    #[tokio::test]
    async fn test_get_coin_price() {
        let coins: Vec<CoinType> = vec![
            CoinType::USDC,
            CoinType::BTC,
            CoinType::ETH,
            CoinType::USDT,
            CoinType::DW20,
        ];
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        for coin in coins {
            let price = fees_cli.get_coin_price_custom(&coin).await.unwrap();
            println!("{}: price {} ", coin, price);
        }
    }

    #[tokio::test]
    async fn test_toos_get_users_tx() {
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let res = fees_cli.get_user_txs("25f1fd7f.local").await;
        println!("_____{:#?}",res);
    }
}
