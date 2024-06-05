use common::utils::math::coin_amount::raw2display;
use common::utils::math::hex_to_bs58;
use near_crypto::SecretKey;
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
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct User {
    pub account_id: String, //用户id
    pub btc_address: String,   //btc地址
    pub btc_level: u8,         //btc等级
    pub dw20: u128,
    pub cly: u128,
    pub is_real_name: bool,        //是否实名
    pub ref_account_id: String, // 上级
    pub last_change_ref_time: u64, //上一次更新上级时间
    pub create_dw20: u64,          // 创建时间
    pub create_cly: u64,    
}

pub struct Airdrop {}
impl ContractClient<Airdrop> {
    //todo: config
    pub async fn new_update_cli() -> Result<Self> {
        //let contract = &common::env::CONF.fees_call_contract;
        let contract = "airdrop0003.chainless";
        Self::gen_cli(contract).await
    }

    pub async fn new_query_cli() -> Result<Self> {
        //let contract = &common::env::CONF.multi_sig_contract;
        let contract = "airdrop0003.chainless";
        Self::gen_cli_without_relayer(contract).await
    }

    //claion after kyc
    pub async fn claim_dw20(
        &self,
        account_id: &str,
        predecessor_account_id: &str,
        btc_address: Option<&str>,
        btc_level: u8,
    ) -> Result<String> {
        let args_str = json!({
            "account_id":  account_id,
            "ref_account_id":  predecessor_account_id,
            "btc_address":  btc_address,
            "btc_level":  btc_level,
        })
        .to_string();
        self.commit_by_relayer("claim_dw20", &args_str).await
    }

    //claim anonymously
    pub async fn claim_cly(&self, account_id: &str) -> Result<String> {
        let args_str = json!({
            "account_id":  account_id,
        })
        .to_string();
        self.commit_by_relayer("claim_cly", &args_str).await
    }

    pub async fn change_predecessor(
        &self,
        account_id: &str,
        predecessor_account_id: &str,
    ) -> Result<String> {
        let args_str = json!({
            "account_id":  account_id,
            "ref_account_id":  predecessor_account_id,
        })
        .to_string();
        self.commit_by_relayer("change_ref", &args_str).await
    }

    pub async fn get_user(
        &self,
        account_id: &str
    ) -> Result<Option<User>> {
        let args_str = json!({
            "account_id":  account_id
        })
        .to_string();
        self.query_call("get_user", &args_str).await
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_ca_airdrop_get_user() {
        let cli = ContractClient::<Airdrop>::new_update_cli().await.unwrap();
        let user_info = cli.get_user("faa80e44.local2").await.unwrap();
        println!("sys_info {:?} ", user_info);
    }
}
