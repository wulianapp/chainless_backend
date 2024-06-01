use common::constants::BRIDGE_DEPOSIT_EXPIRE_TIME;
use common::data_structures::bridge::OrderType as BridgeOrderType;
use common::encrypt::bs58_to_hex;
use near_crypto::SecretKey;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, Balance, BlockReference, Finality, FunctionArgs};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, warn};

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
use crate::relayer::MULTI_SIG_RELAYER_POOL;
use crate::ContractClient;
use anyhow::Result;

use common::utils::time::*;
use ethers::types::transaction::eip712::Eip712;
use ethers::{abi::Address, types::Signature};
use ethers_contract::{Eip712, EthAbiType};
use ethers_core::types::U256;
use ethers_signers::{LocalWallet, Signer};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SignedOrder {
    pub number: u64,
    pub signer: AccountId,
    pub signer_type: u32, //0 syncless 1 signature
    pub signature: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Status {
    Syncless,
    Default,
    Pending,
    Signed,
    Completed,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct BridgeOrder {
    pub chain_id: u128,              //外链id
    pub order_type: BridgeOrderType, //Withdraw,Deposit
    pub account_id: AccountId,       //无链id
    pub symbol: String,              //代币符号
    pub amount: u128,                //
    pub address: String,             //外链地址
    pub signers: Vec<SignedOrder>,   //签名详情
    pub signature: Option<String>,   //充值签名
    pub status: Status,              //订单状态
    pub block_number: u64,           //订单状态
    pub txhash: Option<String>,      //创建时间
    pub create_at: u64,              //更新时间
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
    contract: Address,
    deadline: U256,
}

pub struct Bridge {}

impl ContractClient<Bridge> {
    pub async fn new_update_cli() -> Result<Self> {
        let contract = &common::env::CONF.bridge_near_contract;
        Self::gen_cli(contract).await
    }

    pub async fn new_query_cli() -> Result<Self> {
        let contract = &common::env::CONF.bridge_near_contract;
        Self::gen_cli_without_relayer(contract).await
    }

    fn eth_contract() -> String {
        common::env::CONF.bridge_eth_contract.clone()
    }

    fn eth_admin_prikey() -> String {
        common::env::CONF.bridge_admin_prikey.clone()
    }

    pub async fn send_coin(&self, receiver: &str, amount: u128) -> Result<String> {
        let receiver: AccountId = AccountId::from_str(receiver)?;
        let args_str = json!({
            "receiver_id":  receiver,
            "amount": amount.to_string(),
        })
        .to_string();
        self.commit_by_relayer("ft_transfer", &args_str).await
    }

    //登陆状态-》服务器签名-》eth用户再签-》 服务器验证签名-》绑定
    pub async fn sign_bind_info(&self, near_account_id: &str, eth_addr: &str) -> Result<String> {
        let data = BindAddress {
            cid: U256::from(now_millis()),
            chainless_id: near_account_id.parse()?,
            owner: eth_addr.parse()?,
            contract: Self::eth_contract().parse()?,
        };

        let bs58_prikey = MULTI_SIG_RELAYER_POOL[0].lock().await.signer.secret_key.to_string();
        let hex_prikey = bs58_to_hex(&bs58_prikey)?;
        let bytes_prikey = hex::decode(hex_prikey)?;
        let prikey = &bytes_prikey[..32];

        let wallet = LocalWallet::from_bytes(prikey)?;
        let signature = format!("0x{}", wallet.sign_typed_data(&data).await?);

        //let decoded = data.encode_eip712().unwrap();
        //let sign = Signature::from_str(&signature).unwrap();
        //let _ad = sign.recover(decoded).unwrap();
        Ok(signature)
    }

    pub async fn sign_bind_eth_addr_info(
        &self,
        near_account_id: &str,
        eth_addr: &str,
    ) -> Result<String> {
        let data = BindAddress {
            cid: U256::from(1500),
            chainless_id: near_account_id.parse()?,
            owner: eth_addr.parse()?,
            contract: Self::eth_contract().parse()?,
        };
        let bs58_prikey = MULTI_SIG_RELAYER_POOL[0].lock().await.signer.secret_key.to_string();
        let hex_prikey = bs58_to_hex(&bs58_prikey)?;
        let bytes_prikey = hex::decode(hex_prikey)?;
        let prikey = &bytes_prikey[..32];

        let wallet = LocalWallet::from_bytes(prikey)?;
        let signature = format!("0x{}", wallet.sign_typed_data(&data).await?);
        Ok(signature)
    }

    pub fn verify_eth_bind_sign(
        &self,
        eth_addr: &str,
        main_account: &str,
        user_eth_sig: &str,
    ) -> Result<bool> {
        let data = BindAddress {
            cid: U256::from(1500),
            chainless_id: main_account.parse()?,
            owner: eth_addr.parse()?,
            contract: Self::eth_contract().parse()?,
        };

        let decoded = data.encode_eip712()?;
        let sign = Signature::from_str(user_eth_sig)?;
        let ad = sign.recover(decoded)?;
        let address = format!("{:?}", ad);
        let eth_addr = "0xCfA15434634c70297E012068148DA3e35DAEc780";
        if eth_addr.eq_ignore_ascii_case(&address) {
            Ok(true)
        } else {
            warn!(
                "verify_eth_bind_sign failed: addr_input: {},addr_decode {}",
                eth_addr, address
            );
            println!(
                "verify_eth_bind_sign failed: addr_input: {},addr_decode {}",
                eth_addr, address
            );
            Ok(false)
        }
    }

    pub async fn set_user_batch(&self, account_id: &str) -> Result<String> {
        //todo: verify user's ecdsa signature
        let account_ids = HashMap::from([(AccountId::from_str(account_id)?, true)]);
        let args_str = json!({
            "account_ids":  account_ids,
        })
        .to_string();
        self.commit_by_relayer("set_user_batch", &args_str).await
    }

    pub async fn bind_eth_addr(
        &self,
        account_id: &str,
        address: &str,
        sig: &str,
    ) -> Result<String> {
        //todo: verify user's ecdsa signature
        let args_str = json!({
            "chain_id": 1500u128,
            "account_id":  account_id,
            "address": address,
            "signature": sig,
        })
        .to_string();
        self.commit_by_relayer("bind_address", &args_str).await
    }

    pub async fn unbind_eth_addr(&self, account_id: &str, address: &str) -> Result<String> {
        //todo: verify user's ecdsa signature
        let args_str = json!({
            "chain_id":1,
            "account_id":  account_id,
            "address": address,
        })
        .to_string();
        self.commit_by_relayer("unbind_address", &args_str).await
    }

    pub async fn get_binded_eth_addr(&self, account_id: &str) -> Result<Option<String>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({
            "chain_id":1500,
            "account_id": user_account_id,
        })
        .to_string();
        self.query_call("get_address_by_account_id", &args_str)
            .await
    }

    pub async fn get_withdraw_info(&self, order_id: u128) -> Result<Option<String>> {
        let args_str = json!({
            "with_id":order_id,
        })
        .to_string();
        self.query_call("get_with_info", &args_str).await
    }

    pub async fn get_last_withdraw_order_id(&self) -> Result<Option<u128>> {
        let args_str = json!({}).to_string();
        self.query_call("get_with_id", &args_str).await
    }

    pub async fn get_last_deposit_order_id(&self) -> Result<Option<u128>> {
        let args_str = json!({}).to_string();
        //todo: get from eth: deposit_id
        self.query_call("get_last_deposit_id", &args_str).await
    }
    /**
     *
     *
     *     pub create_at: u64,//创建时间
            pub block_number: u64,

    */

    pub async fn list_withdraw_order(
        &self,
        account_id: &str,
    ) -> Result<Option<Vec<(u128, BridgeOrder)>>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({
            "account_id":user_account_id,
            //todo
            "order_type": "Withdraw",
            "chain_id": None::<u128>,
            //"max": self.get_last_withdraw_order_id().await?,
            "page": 1,
            "page_size":10000,
        })
        .to_string();
        self.query_call("list_order", &args_str).await
    }

    pub async fn list_deposit_order(
        &self,
        account_id: &str,
    ) -> Result<Option<Vec<(u128, BridgeOrder)>>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({
            "account_id":user_account_id,
            "order_type": "Deposit",
            "chain_id": None::<u128>,
            "page": 1,
            "page_size":10000,
        })
        .to_string();
        self.query_call("list_order", &args_str).await
    }

    //服务器签名-》eth用户直接锁仓 ---》桥服务端-监控后台mint
    pub async fn sign_deposit_info(
        &self,
        coin: CoinType,
        amount: u128,
        account_id: &str,
    ) -> Result<(String, u64, u64)> {
        let deadline = (now_millis() + BRIDGE_DEPOSIT_EXPIRE_TIME) / 1000;
        let cid = now_millis();
        let amount = if coin == CoinType::ETH {
            U256::zero()
        } else {
            U256::from(amount)
        };
        //todo: 签名的订单只有这个有权限
        let prikey = hex::decode(Self::eth_admin_prikey())?;
        let wallet = LocalWallet::from_bytes(&prikey)?;
        let data = DepositStruct {
            cid: U256::from(cid),
            chainless_id: account_id.parse()?,
            symbol: coin.to_string(),
            amount,
            contract: Self::eth_contract().parse()?,
            deadline: U256::from(deadline),
        };
        println!("{:#?}", data);
        let signature = format!("{}", wallet.sign_typed_data(&data).await?);
        Ok((signature, deadline, cid))
    }
}

#[cfg(test)]
mod tests {

    use common::data_structures::get_support_coin_list;
    use common::utils::math::*;

    use crate::{eth_cli::EthContractClient, multi_sig::MultiSig};

    use super::*;

    fn fake_eth_bind_sign() -> String {
        todo!()
    }

    fn fake_eth_deposit_sign() -> String {
        todo!()
    }

    #[tokio::test]
    async fn test_eth_sign() {
        let bridge_cli = ContractClient::<Bridge>::new_update_cli().await.unwrap();
        let set_res = bridge_cli.set_user_batch("node0").await.unwrap();
        println!("set_user_batch txid {} ", set_res);

        let sig = bridge_cli
            .sign_bind_info("node0", "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F9")
            .await
            .unwrap();
        println!("sign_bind sig {} ", sig);

        //todo: sig on imtoken and verify on server

        let bind_res = bridge_cli
            .bind_eth_addr("node0", "0x52D786dE49Bec1bdfc7406A9aD746CAC4bfD72F9", &sig)
            .await
            .unwrap();
        println!("bind_res {} ", bind_res);

        let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("node0").await;
        println!(
            "current_bind_res {} ",
            current_binded_eth_addr.unwrap().unwrap()
        );

        let sig = bridge_cli
            .sign_deposit_info(CoinType::USDT, 100, "node0")
            .await;
        println!("sign_deposit  {:?} ", sig.unwrap());
    }

    #[tokio::test]
    async fn test_bind_deposit() {
        let bridge_cli = ContractClient::<Bridge>::new_update_cli().await.unwrap();
        let set_res = bridge_cli.set_user_batch("test").await.unwrap();
        println!("set_user_batch txid {} ", set_res);

        let sig = bridge_cli
            .sign_bind_info("test", "0xcb5afaa026d3de65de0ddcfb1a464be8960e334a")
            .await
            .unwrap();
        println!("sign_bind sig {} ", sig);

        //todo: sig on imtoken and verify on server

        if !bridge_cli
            .verify_eth_bind_sign("0xcb5afaa026d3de65de0ddcfb1a464be8960e334c", "test2", &sig)
            .unwrap()
        {
            panic!("1111");
        }

        let bind_res = bridge_cli
            .bind_eth_addr("test", "0xcb5afaa026d3de65de0ddcfb1a464be8960e334a", &sig)
            .await
            .unwrap();
        println!("bind_res {} ", bind_res);

        let (sig, deadline, cid) = bridge_cli
            .sign_deposit_info(CoinType::USDT, 111, "test")
            .await
            .unwrap();
        println!("sign_deposit  {} ", sig);

        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("test2").await;
        println!(
            "current_bind_res {} ",
            current_binded_eth_addr.unwrap().unwrap()
        );

        let cli = EthContractClient::<crate::bridge_on_eth::Bridge>::new().unwrap();
        let deposit_res = cli
            .deposit("test", "usdt", 100000u128, &sig, deadline, cid)
            .await
            .unwrap();
        println!("{:?}", deposit_res);

        let coin_cli = ContractClient::<crate::coin::Coin>::new_update_cli(CoinType::USDT)
            .await
            .unwrap();
        loop {
            let balance = coin_cli.get_balance("test").await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            println!("test_balance_——————{:?}", balance);
        }
    }

    #[tokio::test]
    async fn tools_batch_deposit() {
        let relayer_eth_addr = "0xcb5afaa026d3de65de0ddcfb1a464be8960e334a";
        let current_balance = crate::eth_cli::general::get_eth_balance(relayer_eth_addr)
            .await
            .unwrap();
        println!("current eth balance: {}", current_balance);
        let deposit_amount = 10_000u128 * BASE_DECIMAL; //10k
        let replayer_acccount_id = "test";

        let bridge_cli = ContractClient::<Bridge>::new_update_cli().await.unwrap();
        let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("test").await.unwrap();
        println!("get_binded_eth_addr {:?} ", current_binded_eth_addr);

        let set_res = bridge_cli.set_user_batch("test").await;
        println!("set_user_batch txid {} ", set_res.unwrap());

        let sig = bridge_cli
            .sign_bind_info(replayer_acccount_id, relayer_eth_addr)
            .await
            .unwrap();
        println!("sign_bind sig {} ", sig);

        let bind_res = bridge_cli
            .bind_eth_addr(replayer_acccount_id, relayer_eth_addr, &sig)
            .await
            .unwrap();
        println!("bind_res {} ", bind_res);

        let coins = get_support_coin_list();
        for coin in coins {
            if coin.eq(&CoinType::DW20) {
                continue;
            }
            let deposit_amount = if coin.eq(&CoinType::ETH) {
                BASE_DECIMAL
            } else {
                deposit_amount
            };

            let coin_cli: ContractClient<crate::coin::Coin> =
                ContractClient::<crate::coin::Coin>::new_update_cli(coin.clone())
                    .await
                    .unwrap();
            let balance1: Option<String> = coin_cli.get_balance("test").await.unwrap();
            println!("test_coin_{}_balance1_——————{:?}", coin, balance1);

            let (sig, deadline, cid) = bridge_cli
                .sign_deposit_info(coin.clone(), deposit_amount, replayer_acccount_id)
                .await
                .unwrap();
            println!("sign_deposit  {} ", sig);

            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

            let current_binded_eth_addr = bridge_cli.get_binded_eth_addr("test").await;
            println!(
                "current_binded_eth_addr {:?} ",
                current_binded_eth_addr.unwrap()
            );

            let cli = EthContractClient::<crate::bridge_on_eth::Bridge>::new().unwrap();
            let deposit_res = if coin.eq(&CoinType::ETH) {
                cli.deposit_eth(replayer_acccount_id, deposit_amount, &sig, deadline, cid)
                    .await
                    .unwrap()
            } else {
                cli.deposit(
                    replayer_acccount_id,
                    &coin.to_string(),
                    deposit_amount,
                    &sig,
                    deadline,
                    cid,
                )
                .await
                .unwrap()
            };
            println!("deposit {:?}", deposit_res);

            loop {
                let balance2: Option<String> =
                    coin_cli.get_balance(replayer_acccount_id).await.unwrap();
                println!("test_coin_{}_balance2_——————{:?}", coin, balance2);
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                if balance1.ne(&balance2) {
                    break;
                }
            }
        }
    }
    #[tokio::test]
    async fn tool_list_order() {
        let bridge_cli = ContractClient::<Bridge>::new_update_cli().await.unwrap();
        let orders = bridge_cli
            .list_withdraw_order("25f1fd7f.local")
            .await
            .unwrap();
        println!("orders {:#?}", orders);
    }
}
