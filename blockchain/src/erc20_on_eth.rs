use anyhow::{Ok, Result};

use ::common::data_structures::CoinType;
use common::env::CONF as ENV_CONF;
use ethers::prelude::*;
use ethers::types::Address;

use std::marker::PhantomData;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::Arc;

use crate::eth_cli::EthContractClient;

#[derive(Clone)]
pub struct Erc20 {}

abigen!(
    Erc20CA,
    "./src/erc20.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

impl EthContractClient<Erc20> {
    pub fn new(coin:&CoinType) -> EthContractClient<Erc20> {
        let coin_erc20_ca = coin.erc20_ca().unwrap();
        let ca: H160 = Address::from_str(&coin_erc20_ca).unwrap();
        //addr: cb5afaa026d3de65de0ddcfb1a464be8960e334a
        //此处的私钥仅仅是做测试用，正常流程中不需要提用户签名上eth链的情况
        let prikey = "e05eb9eb3223d310252755e1c2fd65d03a3f9b45955186b4bea78c292cdcaa2b";
        let wallet = prikey
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(1500u32);
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/").unwrap();

        let cli = Arc::new(SignerMiddleware::new(provider, wallet));
        EthContractClient {
            client: cli,
            contract_addr: ca,
            phantom: PhantomData,
        }
    }

    pub async fn relayer_approve(
        &self,
        spender: &str,
        amount: u128,
    ) -> Result<TransactionReceipt> {
        let spender = Address::from_str(spender).unwrap();
        let coin_ca = self.contract_addr;
        let amount = U256::from(amount);

        let erc20_cli = Erc20CA::new(coin_ca, self.client.clone());
        let approve_res = erc20_cli
            .approve(spender, amount)
            .legacy()
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
        println!("send_res {:?}", approve_res.as_ref().unwrap());
        Ok(approve_res.unwrap())
    }

    pub async fn balance_of(&self, address: &str) -> Result<u128> {
        let address = Address::from_str(address).unwrap();
        //todo: coin address from config
        let coin_cli = Erc20CA::new(self.contract_addr, self.client.clone());
        let balance = coin_cli.balance_of(address).call().await.unwrap();
        Ok(balance.as_u128())
    }

    pub async fn allowance(&self, owner: &str, spender: &str) -> Result<u128> {
        let owner = Address::from_str(owner).unwrap();
        let spender = Address::from_str(spender).unwrap();

        let coin_cli = Erc20CA::new(self.contract_addr, self.client.clone());
        let allow_amount = coin_cli.allowance(owner, spender).call().await.unwrap();
        Ok(allow_amount.as_u128())
    }
}

#[cfg(test)]
mod tests {

    use ::common::data_structures::{
        get_support_coin_list, get_support_coin_list_without_cly, CoinType,
    };

    use super::*;

    #[tokio::test]
    async fn test_erc20_op() {
        let cli = EthContractClient::<Erc20>::new(&CoinType::USDT);
        let address = "0xcb5afaa026d3de65de0ddcfb1a464be8960e334a";
        let relayer_addr = "cb5afaa026d3de65de0ddcfb1a464be8960e334a";

        let balance = cli.balance_of(address).await.unwrap();
        println!("balance__{}", balance);
        let spender = hex::encode(cli.contract_addr);
        let amount = 112 * 10u128.pow(18);
        let _approve_res = cli
            .relayer_approve(&spender, amount)
            .await
            .unwrap();
        let allow_amount = cli
            .allowance(relayer_addr, &spender)
            .await
            .unwrap();
        println!("allow_amount__{}", allow_amount);
    }
    #[tokio::test]
    async fn tools_batch_approve() {
        let address = "0xcb5afaa026d3de65de0ddcfb1a464be8960e334a";
        let relayer_addr = "cb5afaa026d3de65de0ddcfb1a464be8960e334a";

        let coins = get_support_coin_list();
        for coin in coins {
            if coin.eq(&CoinType::ETH) || coin.eq(&CoinType::DW20) {
                continue;
            }
            let cli = EthContractClient::<Erc20>::new(&coin);
            let balance = cli.balance_of(address).await.unwrap();
            println!("coin {} balance__{}", coin.to_string(), balance);
            let spender = hex::encode(cli.contract_addr);
            let amount = 10000000000000000 * 10u128.pow(18);
            
            let _approve_res = cli
                .relayer_approve( &spender, amount)
                .await
                .unwrap();
            
            let allow_amount = cli
                .allowance( relayer_addr, &spender)
                .await.unwrap();
            println!("coin: {} ,allow_amount__{}",coin,allow_amount);
        }
    }
}
