#![feature(let_chains)]

mod general;
mod airdrop;
mod newbie_reward;
mod hello;
pub mod coin;

use lazy_static::lazy_static;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use std::marker::PhantomData;
use near_crypto::{InMemorySigner, SecretKey};
use near_primitives::types::AccountId;



lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://120.232.251.101:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}


#[derive(Clone)]
struct ContractClient<T>{
    pub deployed_at: AccountId,
    relayer: InMemorySigner,
    phantom: PhantomData<T>,
}

pub async  fn test1(){
    let mainnet_client = JsonRpcClient::connect("http://120.232.251.101:8061");
    let tx_status_request = methods::tx::RpcTransactionStatusRequest {
        transaction_info: TransactionInfo::TransactionId {
            hash: "2HvMg8EpsgweGFSG87ngpJ97yWnuX9nBNB9yaXn8HC8w".parse().unwrap(),
            account_id: "node0".parse().unwrap(),
        },
    };

// call a method on the server via the connected client
    let tx_status = mainnet_client.call(tx_status_request).await.unwrap();

    println!("{:?}", tx_status);
}
/***
todo:
1、airdrop interface
2、newbie awards
3、mpc combine
4、transaction broadcast
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_client_sdk(){
      test1().await;
    }
}
