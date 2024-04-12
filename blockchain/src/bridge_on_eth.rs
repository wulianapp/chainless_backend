use anyhow::{Ok, Result};

use common::env::CONF as ENV_CONF;
use ethers::prelude::*;
use ethers::types::Address;

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
pub struct TokenInfo{
    pub symbol: String,
    pub chainless: String,
    pub coin_addr:String,
    pub allow:bool,
}


#[derive(Debug)]
pub struct DepositOrderInfo{
    pub cid: String,
    pub deposit_id: String,
    pub chainless_id:String,
    pub user:String,
    pub amount:u128,
    pub symbol:String,
    pub timestamp:u128,
    pub signature:String,
}

impl EthContractClient<Bridge> {
    pub fn new() -> EthContractClient<Bridge> {
        let ca = Address::from_str("0x4a9B370a2Bb04E1E0D78c928254a4673618FD73f").unwrap();
        //addr: cb5afaa026d3de65de0ddcfb1a464be8960e334a
        let prikey = "e05eb9eb3223d310252755e1c2fd65d03a3f9b45955186b4bea78c292cdcaa2b";
        let wallet = prikey.parse::<LocalWallet>().unwrap().with_chain_id(1500u32);
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/").unwrap();

        let cli = Arc::new(SignerMiddleware::new(provider, wallet));
        EthContractClient {
            client: cli,
            contract_addr: ca,
            phantom: PhantomData,
        }
    }

    pub async fn deposit(
        &self,
        chainless_id: &str,
        symbol: &str,
        amount: u128,
        signature: &str,
        deadline: u128,
    ) -> Result<TransactionReceipt> {
        let cid = U256::from(1u32);
        let amount = U256::from(amount);
        let deadline = U256::from(deadline);
        let signature = hex::decode(signature).unwrap();
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());
        let send_res = bridge_cli.deposit(
            cid, 
            chainless_id.to_owned(),
             symbol.to_owned(),
              amount, 
              signature.into(), 
              deadline
        ).legacy().send().await.unwrap().await.unwrap();
        println!("send_res {:?}",send_res.as_ref().unwrap());  
        Ok(send_res.unwrap())
    }


    pub async fn token_info(
        &self,
        symbol: &str,
    ) -> Result<TokenInfo> {
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());
        let (symbol,chainless,coin_addr,allow) = bridge_cli.token_info(
             symbol.to_owned()
        ).call().await.unwrap();

        Ok(TokenInfo{
            symbol,
            chainless,
            coin_addr: coin_addr.to_string(),
            allow,
        })
    }

    pub async fn get_deposit_order_by_id(
        &self,
        id: u32,
    ) -> Result<DepositOrderInfo> {
        let id = U256::from(id);
        let bridge_cli = BridgeCA::new(self.contract_addr.clone(), self.client.clone());
        let (cid,deposit_id,chainless_id,user,amount,symbol,timestamp,signature,test) 
            = bridge_cli.deposit_info(id).call().await.unwrap();

        Ok(DepositOrderInfo{
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
    async fn test_get_token(){
        let cli = EthContractClient::<Bridge>::new();
        let token = cli.token_info("usdt").await.unwrap();
        println!("{:?}",token);
    }
}