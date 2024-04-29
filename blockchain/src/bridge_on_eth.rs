use anyhow::{Ok, Result};

use ::common::data_structures::CoinType;
use ::common::utils::time::now_millis;
use common::env::CONF as ENV_CONF;
use ethers::prelude::*;
use ethers::types::Address;
use hex::FromHex;

use std::marker::PhantomData;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::Arc;

use crate::eth_cli::EthContractClient;

#[derive(Clone)]
pub struct Bridge {}

abigen!(
    BridgeCA,
    "./src/bridge.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[derive(Debug)]
pub struct TokenInfo {
    pub symbol: String,
    pub chainless: String,
    pub coin_addr: String,
    pub allow: bool,
}

#[derive(Debug)]
pub struct DepositOrderInfo {
    pub cid: String,
    pub deposit_id: String,
    pub chainless_id: String,
    pub user: String,
    pub amount: u128,
    pub symbol: String,
    pub timestamp: u128,
    pub signature: String,
}

impl EthContractClient<Bridge> {
    pub fn new() -> Result<EthContractClient<Bridge>> {
        let ca = Address::from_str(&ENV_CONF.bridge_eth_contract)?;
        //addr: cb5afaa026d3de65de0ddcfb1a464be8960e334a
        let prikey = "e05eb9eb3223d310252755e1c2fd65d03a3f9b45955186b4bea78c292cdcaa2b";
        let wallet = prikey
            .parse::<LocalWallet>()?
            .with_chain_id(1500u32);
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/").unwrap();

        let cli = Arc::new(SignerMiddleware::new(provider, wallet));
        Ok(EthContractClient {
            client: cli,
            contract_addr: ca,
            phantom: PhantomData,
        })
    }

    pub async fn deposit(
        &self,
        chainless_id: &str,
        symbol: &str,
        amount: u128,
        signature: &str,
        deadline: u64,
        cid: u64,
    ) -> Result<TransactionReceipt> {
        let cid = U256::from(cid);
        let amount = U256::from(amount);
        let deadline = U256::from(deadline);
        let signature = hex::decode(signature).unwrap();
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());

        println!(
            "cid {}\n,chainless_id {}\n,symbol {}\n,amount {}\n,signature {}\n,deadline {}\n",
            cid,
            chainless_id,
            symbol,
            amount,
            hex::encode(signature.clone()),
            deadline
        );
        let send_res = bridge_cli
            .deposit(
                cid,
                chainless_id.to_owned(),
                symbol.to_owned(),
                amount,
                signature.into(),
                deadline,
            )
            .legacy()
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
        println!("send_res {:?}", send_res.as_ref().unwrap());
        Ok(send_res.unwrap())
    }

    pub async fn deposit_eth(
        &self,
        chainless_id: &str,
        amount: u128,
        signature: &str,
        deadline: u64,
        cid: u64,
    ) -> Result<TransactionReceipt> {
        let cid = U256::from(cid);
        let amount = U256::from(amount);
        let deadline = U256::from(deadline);
        let signature: Vec<u8> = hex::decode(signature).unwrap();
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());

        println!(
            "cid {}\n,chainless_id {}\n,symbol {}\n,amount {}\n,signature {}\n,deadline {}\n",
            cid,
            chainless_id,
            CoinType::ETH,
            amount,
            hex::encode(signature.clone()),
            deadline
        );
        let send_res = bridge_cli
            .deposit(
                cid,
                chainless_id.to_owned(),
                CoinType::ETH.to_string(),
                U256::zero(),
                signature.into(),
                deadline,
            )
            .value(U256::from(amount))
            .legacy()
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
        println!("send_res {:?}", send_res.as_ref().unwrap());
        Ok(send_res.unwrap())
    }

    pub async fn withdraw(
        &self,
        order_id: u128,
        chainless_id: &str,
        amount: u128,
        symbol: &str,
        signature: &str,
    ) -> Result<TransactionReceipt> {
        let amount = U256::from(amount);
        let signature = signature.replace("0x", "");
        let signature = hex::decode(signature).unwrap();
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());

        println!(
            "order_id {}\n,chainless_id {}\n,symbol {}\n,amount {}\n,signature {}\n",
            order_id,
            chainless_id,
            symbol,
            amount,
            hex::encode(signature.clone())
        );
        let send_res = bridge_cli
            .withdraw(
                U256::from(order_id),
                chainless_id.to_owned(),
                U256::from(amount),
                symbol.to_owned(),
                vec![signature.into()],
            )
            .legacy()
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
        println!("send_res {:?}", send_res.as_ref().unwrap());
        Ok(send_res.unwrap())
    }

    pub async fn token_info(&self, symbol: &str) -> Result<TokenInfo> {
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());
        let (symbol, chainless, coin_addr, allow) = bridge_cli
            .token_info(symbol.to_owned())
            .call()
            .await
            .unwrap();

        Ok(TokenInfo {
            symbol,
            chainless,
            coin_addr: coin_addr.to_string(),
            allow,
        })
    }

    pub async fn get_deposit_order_by_id(&self, id: u32) -> Result<DepositOrderInfo> {
        let id = U256::from(id);
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());
        let (cid, deposit_id, chainless_id, user, amount, symbol, timestamp, signature, test) =
            bridge_cli.deposit_info(id).call().await.unwrap();

        Ok(DepositOrderInfo {
            cid: cid.to_string(),
            deposit_id: deposit_id.to_string(),
            chainless_id,
            user: user.to_string(),
            amount: amount.as_u128(),
            symbol,
            timestamp: timestamp.as_u128(),
            signature: hex::encode(signature.to_vec()),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_get_token() {
        let cli = EthContractClient::<Bridge>::new().unwrap();
        let token = cli.token_info("usdt").await.unwrap();
        println!("{:?}", token);
    }
}
