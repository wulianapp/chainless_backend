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
use anyhow::{Result};

//air100010
pub struct Airdrop {}
impl ContractClient<Airdrop> {
    //todo: config
    pub async fn new() -> Result<Self> {
        //let contract = &common::env::CONF.fees_call_contract;
        let contract = "airdrop0003.chainless";
        Self::gen_signer(contract).await
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
    pub async fn claim_cly(
        &self,
        account_id: &str,
    ) -> Result<String> {
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
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_get_sys_info() {
        let cli = ContractClient::<Airdrop>::new().await.unwrap();
        //let sys_info = cli.get_sys_info().await.unwrap();
        //println!("sys_info {:?} ", sys_info);
    }
}
