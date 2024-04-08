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
use anyhow::Result;


use ethers::types::transaction::eip712::Eip712;
use ethers::{abi::Address, types::Signature};
use ethers_contract::{Eip712, EthAbiType};
use ethers_core::types::U256;
use ethers_signers::{LocalWallet, Signer};


pub struct SignedOrder {
    pub number: u64,
    pub signer: AccountId,
    pub signature: Option<String>,
}

pub enum Status {
    Default,
    Pending,
    Signed
}

pub enum OrderType {
    Withdraw,
    Deposit
}


struct BridgeOrder {
    pub chain_id: u128,//外链id
    pub order_type: OrderType,//Withdraw,Deposit
    pub account_id: AccountId,//无链id
    pub symbol: String,//代币符号
    pub amount: u128,//
    pub address: String,//外链地址
    pub signers: Vec<SignedOrder>,//签名详情
    pub signature: Option<String>,//充值签名
    pub status: Status,//订单状态
    pub create_at: u64,//创建时间
    pub update_at: u64,//更新时间
}


#[derive(Serialize, Deserialize, Clone, Eip712, EthAbiType, Debug)]
#[eip712(
    name = "Eip712Vault",
    version = "1",
    chain_id = 1,
    salt = "eip712-vault-23x8Dek33kgD"
)]
struct BindAddress {
    cid: U256,
    chainless_id: String,
    owner: Address,
    contract: Address,
}


#[derive(Serialize, Deserialize, Clone, Eip712, EthAbiType, Debug)]
#[eip712(
    name = "Eip712Vault",
    version = "1",
    chain_id = 1,
    salt = "eip712-vault-23x8Dek33kgD"
)]
struct DepositSign {
    cid: U256,
    chainless_id: String,
    symbol: String,
    amount: U256,
    owner: Address,
    contract: Address,
    deadline: U256
}


pub struct Bridge {}
impl ContractClient<Bridge> {
    //fixme: gen once object
    pub fn new() -> Result<Self> {
        let prikey_str = &common::env::CONF.multi_sig_relayer_prikey;
        //cvault0001.chainless
        let contract = &common::env::CONF.bridge_near_contract;
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

    pub async fn send_coin(&self, receiver: &str, amount: u128) -> Result<String> {
        let receiver: AccountId = AccountId::from_str(receiver).unwrap();
        let args_str = json!({
            "receiver_id":  receiver,
            "amount": amount.to_string(),
        })
        .to_string();
        self.commit_by_relayer("ft_transfer", &args_str).await
    }

    /*** 
    pub async fn get_deposit_order(&self, order_id: &str) -> Result<Option<String>> {
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({"account_id": user_account_id}).to_string();
        self.query_call("ft_balance_of", &args_str).await
    }

    pub async fn get_withdraw_order(&self, order_id: &str) -> Result<Option<String>> {
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({"account_id": user_account_id}).to_string();
        self.query_call("ft_balance_of", &args_str).await
    }
*/
    //登陆状态-》服务器签名-》eth用户再签-》 服务器验证签名-》绑定
    pub async fn sign_bind_info(&self,near_account_id:&str,eth_addr:&str) -> String{
        let data = BindAddress {
            cid: U256::from(1),
            chainless_id: near_account_id.parse().unwrap(),
            owner: eth_addr.parse().unwrap(),
            contract: "0x91341BA63f81c5F1C2879f108645f3a8Bd6051c5"
                .parse()
                .unwrap(),
        };
        let prikey = self.relayer.secret_key.unwrap_as_ed25519().0;
        let prikey = &prikey[..32];
        let wallet = LocalWallet::from_bytes(prikey).unwrap();
        let signature = format!("0x{}", wallet.sign_typed_data(&data).await.unwrap());
        println!("signature: {}", signature);

        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("addr--- {}", format!("{:?}", ad));
        signature
    }


    pub async fn bind_eth_addr(&self,account_id:&str,address:&str) -> Result<String>{
        //todo: verify user's ecdsa signature
        let args_str = json!({
            "chain_id":1,
            "account_id":  account_id,
            "address": address,
        })
        .to_string();
        self.commit_by_relayer("bind_address", &args_str).await
    }


    pub async fn unbind_eth_addr(&self,account_id:&str,address:&str) -> Result<String>{
        //todo: verify user's ecdsa signature
        let args_str = json!({
            "chain_id":1,
            "account_id":  account_id,
            "address": address,
        })
        .to_string();
        self.commit_by_relayer("unbind_address", &args_str).await
    }


    pub async fn get_binded_eth_addr(&self,account_id:&str) -> Result<Option<String>>{
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "chain_id":1,
            "account_id": user_account_id,
        }).to_string();
        self.query_call("get_address_by_account_id", &args_str).await
    }

    //服务器签名-》eth用户直接锁仓 ---》桥服务端-监控后台mint
    pub async fn sign_deposit_info(&self,coin:CoinType,amount:u128,user_id:&str) -> Result<String>{
        let owner = self.get_binded_eth_addr(user_id).await.unwrap().unwrap();
        println!("owner {}",owner);
        let prikey = self.relayer.secret_key.unwrap_as_ed25519().0;
        let wallet = LocalWallet::from_bytes(&prikey).unwrap();
        let data = DepositSign {
            cid: U256::from(1),
            chainless_id: user_id.parse().unwrap(),
            symbol: coin.to_string(),
            amount: U256::from(amount),
            owner: owner.parse().unwrap(),
            contract: "0x91341BA63f81c5F1C2879f108645f3a8Bd6051c5"
                .parse()
                .unwrap(),
            deadline: U256::from(1)
        };

        let signature = format!("0x{}", wallet.sign_typed_data(&data).await.unwrap());
        println!("signature: {}", signature);

        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("addr--- {}", format!("{:?}", ad));
        Ok(signature)
    }

  
    //在多签转账，创建提现订单
    //fn new_order(chain_id: u128, account_id: AccountId, amount: u128, token: AccountId)
    fn create_order(){

    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn fake_eth_bind_sign() -> String{
        todo!()
    }


    fn fake_eth_deposit_sign() -> String{
        todo!()
    }

    #[tokio::test]
    async fn test_eth_sign() {
        let bridge_cli = ContractClient::<Bridge>::new().unwrap();
        let sig = bridge_cli.sign_bind_info(
             "user.node0",
              "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F9"
            ).await;
        println!("sig {} ",sig);

        let bind_res = bridge_cli.bind_eth_addr(
            "user.node0",
        "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F9"
        ).await.unwrap();
        println!("bind_res {} ",bind_res);


        let sig = bridge_cli.sign_deposit_info(
            CoinType::BTC,
            100,
            "user.node0"
           ).await;
       println!("sig {} ",sig.unwrap());
    }
}