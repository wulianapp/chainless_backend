
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
use anyhow::{Ok, Result};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct U128(pub u128);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NetUsers {
    pub user_receive_dw20: u64, //领取dw20人数
    pub user_receive_cly: u64, //领取cly人数
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SettleTimesU64 {
    pub three: u64,
    pub nine: u64,
    pub twenty_one: u64,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SysInfo{
    pub net_users: NetUsers,
    pub admin: Vec<(String,bool)>,
    pub settle_times: SettleTimesU64,//3,9,21,top排行榜已结算时间
    pub next_settle_times: SettleTimesU64,
    pub start_times: u64,// 合约开始时间
    pub fire_times: u64,//点火时间，控制开始释放时间
    pub free_times: u64, //全局释放至时间
    pub free_off: bool, //仅控制释放操作是否允许
    pub disuse_times: u64, //下次淘汰时间
    pub times_elapsed: u64, //用于控制合约时间 用于测试
    pub free_total_token: Vec<(String, u128)> //释放token总和
}

pub struct AirReward {}
impl ContractClient<AirReward> {
    //fixme: gen once object
    pub fn new() -> Result<Self> {
        let prikey_str = &common::env::CONF.multi_sig_relayer_prikey;
        let relayer_account = &common::env::CONF.multi_sig_relayer_account_id;
        let prikey_str= "ed25519:2zGt1i93avrks4RGeYXw7WvaoWmBWz4PcjWpTmqCRWFU4irviDjPvWCtTi14Wsz8DKaLysAeUBfYtyn8qovMGeNz";
        let relayer_account = "chainless";

        //cvault0001.chainless
        let contract = &common::env::CONF.fees_call_contract;
        let contract = "air100010";
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

    pub async fn get_sys_info(
        &self
    ) -> Result<Option<SysInfo>> {
        //todo: verify user's ecdsa signature
        let args_str = json!({})
        .to_string();
        self.query_call("get_sys_info", &args_str).await
    }

    pub async fn get_block_times(&self) -> Result<Option<u64>> {
        let args_str = json!({})
        .to_string();
        self.query_call("get_block_times", &args_str).await
    }

    pub async fn get_reward_start_times(&self) -> Result<Option<u64>> {
        let args_str = json!({})
        .to_string();
        self.query_call("get_reward_start_times", &args_str).await
    }

    pub async fn get_next_zero_sec(&self) -> Result<Option<u64>> {
        let args_str = json!({})
        .to_string();
        self.query_call("get_next_zero_sec", &args_str).await
    }


    pub async fn get_net_users(&self) -> Result<Option<NetUsers>> {
        let args_str = json!({})
        .to_string();
        self.query_call("get_next_zero_sec", &args_str).await
    }

    //
    pub async fn is_invite_code_valid(&self,code:&str) -> Result<bool> {
        let args_str = json!({
            "code":code
        })
        .to_string();
        self.query_call("get_next_zero_sec", &args_str).await.map(|x| x.unwrap())
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
    async fn test_get_sys_info() {
        let cli = ContractClient::<AirReward>::new().unwrap();        
        let sys_info = cli.get_sys_info().await.unwrap();
        println!("sys_info {:?} ", sys_info);
    }
}
