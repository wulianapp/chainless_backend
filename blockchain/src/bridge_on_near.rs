use near_crypto::SecretKey;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality, FunctionArgs};
use std::collections::HashMap;
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


#[derive(Deserialize, Serialize, Debug, PartialEq,Clone)]
pub struct SignedOrder {
    pub number: u64,
    pub signer: AccountId,
    pub signature: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq,Clone)]
pub enum Status {
    Default,
    Pending,
    Signed
}
#[derive(Deserialize, Serialize, Debug, PartialEq,Clone)]
pub enum OrderType {
    Withdraw,
    Deposit
}

#[derive(Deserialize, Serialize, Debug, PartialEq,Clone)]
pub struct BridgeOrder {
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
struct DepositStruct {
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
        println!("___{}__{}",prikey_str,contract);
        let pri_key: SecretKey = prikey_str.parse()?;
        let pubkey = get_pubkey(&pri_key.to_string())?;

        let account_id = AccountId::from_str(&pubkey)?;
        let relayer_account = &common::env::CONF.multi_sig_relayer_account_id;
        println!("0002___{}",prikey_str);
       let account_id = AccountId::from_str(relayer_account)?;
        println!("0003___{}",account_id);

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
            cid: U256::from(1500),
            chainless_id: near_account_id.parse().unwrap(),
            owner: eth_addr.parse().unwrap(),
            contract: "0x4a9B370a2Bb04E1E0D78c928254a4673618FD73f"
                .parse()
                .unwrap(),
        };
        let prikey = self.relayer.secret_key.unwrap_as_ed25519().0;
        let prikey = &prikey[..32];
        println!("prikey: {}", hex::encode(prikey));
        let wallet = LocalWallet::from_bytes(prikey).unwrap();
        let signature = format!("0x{}", wallet.sign_typed_data(&data).await.unwrap());
        println!("signature: {}", signature);

        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("addr--- {}", format!("{:?}", ad));
        signature
    }

    pub async fn sign_bind_eth_addr_info(&self,near_account_id:&str,eth_addr:&str) -> String{
        let data = BindAddress {
            cid: U256::from(1),
            chainless_id: near_account_id.parse().unwrap(),
            owner: eth_addr.parse().unwrap(),
            contract: "0x91341BA63f81c5F1C2879f108645f3a8Bd6051c1"
                .parse()
                .unwrap(),
        };
        let prikey = self.relayer.secret_key.unwrap_as_ed25519().0;
        let prikey = &prikey[..32];
        println!("prikey: {}", hex::encode(prikey));
        let wallet = LocalWallet::from_bytes(prikey).unwrap();
        let signature = format!("0x{}", wallet.sign_typed_data(&data).await.unwrap());
        signature
    }


    pub fn verify_eth_bind_sign(&self,
        eth_addr:&str,
        main_account:&str,
        user_eth_sig:&str
    ) -> bool{
        let data = BindAddress {
            cid: U256::from(1),
            chainless_id: main_account.parse().unwrap(),
            owner: eth_addr.parse().unwrap(),
            contract: "0x91341BA63f81c5F1C2879f108645f3a8Bd6051c1"
                .parse()
                .unwrap(),
        };

        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&user_eth_sig).unwrap();
        let ad = sign.recover(decoded).unwrap();
        let address = format!("{:?}", ad);
        if eth_addr.eq_ignore_ascii_case(&address){
            true
        }else {
            false
        }
    }

    pub async fn set_user_batch(&self,account_id: &str) -> Result<String>{
        //todo: verify user's ecdsa signature
        let account_ids = HashMap::from([
            (AccountId::from_str(account_id).unwrap(),true)
        ]);
        let args_str = json!({
            "account_ids":  account_ids,
        })
        .to_string();
        self.commit_by_relayer("set_user_batch", &args_str).await
    }

    pub async fn bind_eth_addr(&self,account_id:&str,address:&str,sig:&str) -> Result<String>{
        //todo: verify user's ecdsa signature
        let args_str = json!({
            "chain_id": 1500,
            "account_id":  account_id,
            "address": address,
            "signature": sig,
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
            "chain_id":1500,
            "account_id": user_account_id,
        }).to_string();
        self.query_call("get_address_by_account_id", &args_str).await
    }

    pub async fn get_withdraw_info(&self,order_id:u128) -> Result<Option<String>>{
        let args_str = json!({
            "with_id":order_id,
        }).to_string();
        self.query_call("get_with_info", &args_str).await
    }



    pub async fn get_last_withdraw_order_id(&self) -> Result<Option<u128>>{
        let args_str = json!({}).to_string();
        self.query_call("get_with_id", &args_str).await
    }

    pub async fn get_last_deposit_order_id(&self) -> Result<Option<u128>>{
        let args_str = json!({}).to_string();
        self.query_call("get_last_deposit_id", &args_str).await
    }



    pub async fn list_withdraw_order(&self,account_id:&str) -> Result<Option<(u128,Vec<(u128,BridgeOrder)>)>>{
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "account_id":user_account_id,
            "chain_id": None::<u128>,
            "max": self.get_last_withdraw_order_id().await?,
            "page_size":10000,
        }).to_string();
        self.query_call("list_order", &args_str).await
    }

    pub async fn list_deposit_order(&self,account_id:&str) -> Result<Option<(u128,Vec<(u128,BridgeOrder)>)>>{
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "account_id":user_account_id,
            "chain_id": Some(1500u128),
            "max": self.get_last_deposit_order_id().await?,
            "page_size":10000,
        }).to_string();
        self.query_call("list_order", &args_str).await
    }

    //服务器签名-》eth用户直接锁仓 ---》桥服务端-监控后台mint
    pub async fn sign_deposit_info(&self,coin:CoinType,amount:u128,account_id:&str) -> Result<String>{
        let owner = self.get_binded_eth_addr(account_id).await.unwrap().unwrap();
        println!("owner {}",owner);
        let prikey = self.relayer.secret_key.unwrap_as_ed25519().0;
        let prikey = &prikey[..32];
        //todo: 签名的订单只有这个有权限
        let prikey = hex::decode("6c7d02e6742c673e8c5b9f9e85966a84706c08a6741d84c1467822b6d681d56f").unwrap();
        let wallet = LocalWallet::from_bytes(&prikey).unwrap();
        let data = DepositStruct {
            cid: U256::from(1500),
            chainless_id: account_id.parse().unwrap(),
            symbol: coin.to_string(),
            amount: U256::from(amount),
            owner: owner.parse().unwrap(),
            contract: "0x4a9B370a2Bb04E1E0D78c928254a4673618FD73f"
                .parse()
                .unwrap(),
            deadline: U256::from(2712916794000001u128)
        };
        println!("{:#?}",data);
        let signature = format!("{}", wallet.sign_typed_data(&data).await.unwrap());



        //check sig
        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("signature {} ,check_deposit0sig--- {}", signature,format!("{:?}", ad));

        /*** 
        let signature = "0x070723df5d93e9a32a72fbf8c130400660c2561166bd9e20991f5fbdee22212a4b3536edfc22c3a512c4dfd26d166942b6337bad88fe3a5fc9c3e2a3cd9da7511c".to_string();
        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("signature2 {} ,check_deposit1sig--- {}", signature,format!("{:?}", ad));

        let signature = "070723df5d93e9a32a72fbf8c130400660c2561166bd9e20991f5fbdee22212a4b3536edfc22c3a512c4dfd26d166942b6337bad88fe3a5fc9c3e2a3cd9da7511c".to_string();
        let decoded = data.encode_eip712().unwrap();
        let sign = Signature::from_str(&signature).unwrap();
        let ad = sign.recover(decoded).unwrap();
        println!("signature2 {} ,check_deposit2sig--- {}", signature,format!("{:?}", ad));
        **/

        Ok(signature)
    }

  
    //在多签转账，创建提现订单
    //fn new_order(chain_id: u128, account_id: AccountId, amount: u128, token: AccountId)
    fn create_order(){

    }
}

#[cfg(test)]
mod tests {

    use crate::{eth_cli::EthContractClient, multi_sig::MultiSig};

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
        let set_res = bridge_cli.set_user_batch("node0").await;
       println!("set_user_batch txid {} ",set_res.unwrap());

        let sig = bridge_cli.sign_bind_info(
             "node0",
              "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F9",
            ).await;
        println!("sign_bind sig {} ",sig);

        //todo: sig on imtoken and verify on server

        let bind_res = bridge_cli.bind_eth_addr(
            "node0",
        "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F8",
        &sig
        ).await.unwrap();
        println!("bind_res {} ",bind_res);


        let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("node0").await;
        println!("current_bind_res {} ",current_binded_eth_addr.unwrap().unwrap());


        let sig = bridge_cli.sign_deposit_info(
            CoinType::USDT,
            100,
            "node0"
           ).await;
       println!("sign_deposit  {} ",sig.unwrap());

    }

    #[tokio::test]
    async fn test_bind_deposit() {
        let bridge_cli = ContractClient::<Bridge>::new().unwrap();
        let set_res = bridge_cli.set_user_batch("test2").await;
       println!("set_user_batch txid {} ",set_res.unwrap());

        let sig = bridge_cli.sign_bind_info(
             "test2",
              "0xcb5afaa026d3de65de0ddcfb1a464be8960e334b",
            ).await;
        println!("sign_bind sig {} ",sig);

        //todo: sig on imtoken and verify on server

         /*** 
        let bind_res = bridge_cli.bind_eth_addr(
            "test2",
        "0xcb5afaa026d3de65de0ddcfb1a464be8960e334b",
        &sig
        ).await.unwrap();
        println!("bind_res {} ",bind_res);
        */
        
            
        let sig = bridge_cli.sign_deposit_info(
            CoinType::USDT,
            111,
            "test"
           ).await.unwrap();
       println!("sign_deposit  {} ",sig);
    
       tokio::time::sleep(std::time::Duration::from_millis(3000)).await;   

        let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("test2").await;
        println!("current_bind_res {} ",current_binded_eth_addr.unwrap().unwrap());

        let cli = EthContractClient::<crate::bridge_on_eth::Bridge>::new();
        //let deposit_res = cli.deposit("test","usdt",111u128,&sig,2712916794000001u128).await.unwrap();
        //println!("{:?}",deposit_res);

       

        let coin_cli = ContractClient::<crate::coin::Coin>::new(CoinType::USDT).unwrap();
        loop{
            let balance = coin_cli.get_balance("test").await.unwrap();  
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;   
            println!("test_balance_——————{:?}",balance);   
        }   
       
    }

}