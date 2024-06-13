use ethers::middleware::SignerMiddleware;
use lazy_static::lazy_static;
use near_jsonrpc_client::JsonRpcClient;

//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;

use std::marker::PhantomData;

use ethers_core::k256::ecdsa::SigningKey;

use ethers::prelude::*;
use std::sync::Arc;

lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = {
        println!("+++__{}",common::env::CONF.chain_rpc);
        JsonRpcClient::connect(&common::env::CONF.chain_rpc)
    };
    //e05eb9eb3223d310252755e1c2fd65d03a3f9b45955186b4bea78c292cdcaa2b
    //cb5afaa026d3de65de0ddcfb1a464be8960e334a
}

#[derive(Clone)]
pub struct EthContractClient<E> {
    pub client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    pub contract_addr: H160,
    pub phantom: PhantomData<E>,
}

pub mod general {
    use anyhow::Result;
    use ethers::prelude::*;
    use ethers::{
        providers::{Http, Middleware, Provider},
        types::Address,
    };
    use std::str::FromStr;

    pub async fn get_eth_balance(addr: &str) -> Result<u128> {
        //addr: cb5afaa026d3de65de0ddcfb1a464be8960e334a
        let addr = Address::from_str(addr)?;
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/")?;
        let balance_before = provider.get_balance(addr, None).await?;
        Ok(balance_before.as_u128())
    }

    pub async fn get_current_height() -> Result<u64> {
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/")?;
        let height = provider.get_block_number().await?;
        Ok(height.as_u64())
    }

    pub async fn get_block<T: Into<BlockId> + Send + Sync>(
        height_or_hash: T,
    ) -> Result<Option<Block<H256>>> {
        let provider = Provider::<Http>::try_from("https://test1.chainless.top/node/")?;
        match provider.get_block(height_or_hash).await? {
            None => Ok(None),
            Some(block) => Ok(Some(block)),
        }
    }
}
