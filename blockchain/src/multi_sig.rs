use anyhow::{anyhow, Ok, Result};
use common::data_structures::{get_support_coin_list, PubkeySignInfo};

use common::utils::time::now_millis;
use hex::ToHex;
use lazy_static::lazy_static;
use near_crypto::InMemorySigner;
use near_crypto::Signature;
use near_crypto::{ED25519SecretKey, PublicKey, SecretKey, Signer};
use serde_json::json;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

use near_primitives::types::AccountId;

use common::data_structures::CoinType;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::coin::Coin;
use crate::fees_call::FeesCall;
use crate::general::get_access_key_list;
use crate::general::pubkey_from_hex_str;

use crate::ContractClient;
use common::utils::math::*;

pub struct MultiSig {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRank {
    pub min: u128,
    pub max_eq: u128,
    pub sig_num: u8,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct SubAccConf {
    pub pubkey: String,
    pub hold_value_limit: u128,
}

impl Default for MultiSigRank {
    fn default() -> Self {
        MultiSigRank {
            min: 0,
            max_eq: 1_000_000u128 * BASE_DECIMAL, //one million
            sig_num: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StrategyData {
    pub master_pubkey: String,
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub servant_pubkeys: Vec<String>,
    pub sub_confs: BTreeMap<String, SubAccConf>,
}

impl ContractClient<MultiSig> {
    pub async fn new_update_cli() -> Result<Self> {
        let contract = &common::env::CONF.multi_sig_contract;
        Self::gen_cli(contract).await
    }

    pub async fn new_query_cli() -> Result<Self> {
        let contract = &common::env::CONF.multi_sig_contract;
        Self::gen_cli_without_relayer(contract).await
    }

    //fixeme:
    //用户永远只持有最后一个master的私钥
    //增加key的时候，新key永远不会放在末尾
    pub async fn get_master_pubkey(&self, account_str: &str) -> Result<String> {
        let list = self.get_master_pubkey_list(account_str).await?;
        if list.len() != 1 {
            Err(anyhow!("account have multi key {:?}", list))?;
        }
        Ok(list[0].clone())
    }

    //key列表是定序的,但是不以时间顺序
    pub async fn get_master_pubkey_list(&self, account_str: &str) -> Result<Vec<String>> {
        let list = get_access_key_list(account_str).await?.keys;
        let list = list.iter().map(|key| key.public_key.to_string()).collect();
        Ok(list)
    }

    pub async fn get_single_master_pubkey_list(&self, account_str: &str) -> Result<String> {
        let list = self.get_master_pubkey_list(account_str).await?;
        if list.len() != 1 {
            Err(anyhow!(
                "unnormal account， it's account have more than 1 master"
            ))?;
        }
        Ok(list[0].clone())
    }

    //add_master
    //这里先查询如果已经存在就不加了
    pub async fn add_key(&self, main_account: &str, new_key: &str) -> Result<(String, String)> {
        let master_pubkey = self.get_master_pubkey(main_account).await?;
        let master_pubkey = pubkey_from_hex_str(&master_pubkey)?;
        let main_account = AccountId::from_str(main_account)?;
        self.gen_raw_with_caller(&main_account, &master_pubkey, "add_key", new_key)
            .await
    }

    //后期可以在链底层增加一个直接替换的接口
    pub async fn delete_key(
        &self,
        main_account: &str,
        delete_key: &str,
    ) -> Result<(String, String)> {
        let master_pubkey = self.get_master_pubkey(main_account).await?;
        let master_pubkey = pubkey_from_hex_str(&master_pubkey)?;
        let main_account = AccountId::from_str(main_account)?;
        self.gen_raw_with_caller(&main_account, &master_pubkey, "delete_key", delete_key)
            .await
    }

    pub async fn get_thresholds_and_min_slave_sigs(
        &self,
        account_id: &str,
    ) -> Result<Option<(Vec<u8>, Vec<u128>)>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({"owner": user_account_id}).to_string();
        self.query_call("get_thresholds_and_min_slave_sigs", &args_str)
            .await
    }

    pub async fn get_slaves(&self, account_id: &str) -> Result<Option<(Vec<PublicKey>)>> {
        let user_account_id = AccountId::from_str(account_id)?;
        let args_str = json!({"owner": user_account_id}).to_string();
        self.query_call("get_slaves", &args_str).await
    }

    //todo:
    pub async fn get_strategy(&self, account_id: &str) -> Result<Option<StrategyData>> {
        //let thresholds = self.get_thresholds_and_min_slave_sigs(account_id).await?;
        //let slaves = self.get_slaves(account_id).await?;
        let mut strategy = StrategyData::default();
        strategy.master_pubkey = self.get_master_pubkey(&account_id).await?;
        Ok(Some(strategy))
    }

    pub async fn remove_account_strategy(&mut self, acc: String) -> Result<String> {
        let acc_id = AccountId::from_str(&acc)?;
        let args_str = json!({"acc": acc_id}).to_string();
        self.commit_by_relayer("remove_account_strategy", &args_str)
            .await
    }

    pub async fn register_account(&mut self, account_id: &str, pubkey: &str) -> Result<String> {
        let arg_str = format!("{}@{}", pubkey, account_id);
        self.commit_by_relayer("register_account", &arg_str).await
    }

    pub async fn gen_send_money_raw(
        &self,
        servant_device_sigs: Vec<PubkeySignInfo>,
        from: &str,
        to: &str,
        coin: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> Result<(String, String)> {
        let caller_id = AccountId::from_str(from)?;
        let caller_pubkey_str = self.get_master_pubkey(from).await?;
        let caller_pubkey = pubkey_from_hex_str(&caller_pubkey_str)?;

        let coin_tx = CoinTx {
            from: AccountId::from_str(from)?,
            to: AccountId::from_str(to)?,
            coin_id: coin.to_account_id(),
            amount: transfer_amount,
            expire_at,
            memo: None,
        };
        //todo: remove tx_index;
        let tx_index = now_millis();
        let args_str = json!({
            "tx_index": tx_index,
            "servant_device_sigs": servant_device_sigs,
            "coin_tx": coin_tx,
        })
        .to_string();
        self.gen_raw_with_caller(&caller_id, &caller_pubkey, "send_money", &args_str)
            .await
    }

    pub fn gen_send_money_info(
        &self,
        sender_id: &str,
        receiver_id: &str,
        coin: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> Result<String> {
        let coin_tx_info = CoinTx {
            from: AccountId::from_str(sender_id)?,
            to: AccountId::from_str(receiver_id)?,
            coin_id: coin.to_account_id(),
            amount: transfer_amount,
            expire_at,
            memo: None,
        };
        let coin_tx_json = serde_json::to_string(&coin_tx_info)?;
        let coin_tx_hex_str = hex::encode(coin_tx_json.as_bytes());
        Ok(coin_tx_hex_str)
    }

    //转账给跨链桥
    //todo: 弃用
    pub async fn internal_withdraw(
        &mut self,
        master_sig: PubkeySignInfo,
        servant_sigs: Vec<PubkeySignInfo>,
        from: &str,
        to: &str,
        coin: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> Result<String> {
        let coin_tx = CoinTx {
            from: AccountId::from_str(from)?,
            to: AccountId::from_str(to)?,
            coin_id: coin.to_account_id(),
            amount: transfer_amount,
            expire_at,
            memo: None,
        };

        let args_str = json!({
            "master_sig": master_sig,
            "servant_sigs": servant_sigs,
            "coin_tx": coin_tx,
        })
        .to_string();
        self.commit_by_relayer("internal_withdraw", &args_str).await
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountSignInfo {
    pub account_id: String,
    pub signature: String,
}

impl AccountSignInfo {
    pub fn new(account_id: &str, signature: &str) -> Self {
        Self {
            account_id: account_id.to_owned(),
            signature: signature.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinTx {
    from: AccountId,
    to: AccountId,
    coin_id: AccountId,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawInfo {
    from: AccountId,
    kind: String,
    coin_id: AccountId,
    amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubAccCoinTx {
    coin_id: AccountId,
    amount: u128,
}

lazy_static! {
    static ref MULTI_SIG_CID: AccountId = AccountId::from_str("multi_sig.node0").unwrap();
}

pub fn get_pubkey(pri_key_str: &str) -> Result<String> {
    let secret_key = near_crypto::SecretKey::from_str(pri_key_str)?;
    let pubkey = secret_key.public_key().unwrap_as_ed25519().0;
    //Ok(pubkey.as_slice()[1..].to_vec().encode_hex())
    Ok(pubkey.encode_hex())
}

fn pubkey_from_hex(hex_str: &str) -> Result<PublicKey> {
    println!("pubkey_from_hex {}", hex_str);
    let sender_id = AccountId::from_str(hex_str)?;
    Ok(PublicKey::from_near_implicit_account(&sender_id)?)
}

pub fn sign_data_by_near_wallet2(prikey_str: &str, data_str: &str) -> Result<String> {
    let prikey: SecretKey = prikey_str.parse()?;
    let prikey_bytes = prikey.unwrap_as_ed25519().0;
    let data = hex::decode(data_str)?;

    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string())?;
    let signer_account_id = AccountId::from_str(&main_device_pubkey)?;
    let signer = InMemorySigner::from_secret_key(signer_account_id, near_secret);
    let signature = signer.sign(&data);
    if let Signature::ED25519(sig) = signature {
        let near_sig_bytes = sig.to_bytes();
        Ok(hex::encode(near_sig_bytes))
    } else {
        unreachable!("")
    }
}

pub fn sign_data_by_near_wallet(prikey_bytes: [u8; 64], data: &[u8]) -> Result<String> {
    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string())?;
    let signer_account_id = AccountId::from_str(&main_device_pubkey)?;
    let signer = InMemorySigner::from_secret_key(signer_account_id, near_secret);
    let signature = signer.sign(data);

    if let Signature::ED25519(sig) = signature {
        let near_sig_bytes = sig.to_bytes();
        Ok(hex::encode(near_sig_bytes))
    } else {
        unreachable!("")
    }
}

#[cfg(test)]
mod tests {

    use near_primitives::action::{Action, FunctionCallAction};
    use tracing::{error, info};

    use super::*;

    use crate::ContractClient;
    use common::log::init_logger;

    /***
    pub async fn set_thresholds_and_min_slave_sigs(
        prikey: &str,
        account_id:&str,
        contract:&str
    ){
        let secret_key: SecretKey = prikey.parse().unwrap();
        let signer_account_id = AccountId::from_str(account_id).unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(
            signer_account_id.to_owned(),
            secret_key
        );

        let set_strategy_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "set_thresholds_and_min_slave_sigs".to_string(),
            args: json!({
            "thresholds": Vec::<String>::new(),
            "min_slave_sigs": Vec::<String>::new(),
            })
            .to_string()
            .into_bytes(),
            gas: 300000000000000, // 100 TeraGas
            deposit: None,        //Some(Deposit{ deposit: 0, symbol: None, fee: None }),
        }))];

    }
    **/
    #[tokio::test]
    async fn test_set_thresholds_and_min_slave_sigs() {
        let prikey = "ed25519:YDqZJcyWYeWN3pw6JBLwZtpkjASs5Q9rYUj3tKQyU719SErbrE75rZiXiWL75MhkF67T9wQZDBQHtCZioTZg1Vz";
        let account_id = "eddy.chainless";
        let contract = "multisig_send_mt.chainless";
    }

    fn servant_keys() -> Vec<String> {
        vec![
            "ed25519:s1sw1PXCkHrbyE9Rmg6j18PoUxnhCNZ2CxSPUvvE7dZK9UCEkpTWC1Zy6ZKWvBcAdK8MoRUSdMsduMFRJrRtuGq".to_string(),
            "ed25519:5cNJ9mg3b3VZoiTyimwz3YZhimF5KTDuV1DMU6TMhR1RR3NtXtArxFizDRoRo4kgUJQdQzM1urNxCKbftNhLNN5v".to_string(),
            "ed25519:4D2nFZNxfpCmTBPZhgEGJs2rFeLEe9MhBVNzZyr5XiYL92PnSbYBUbAmPnx4qhi6WQkrFGasNjTdLMNDrj3vRjQU".to_string(),
            "ed25519:vUxMDvDoFVT9qxNZWDpc7TLjK4W8MLGnL6UvardxbcptYtm2VJxaiFq9rZ6LMfxxzs5NVQKpr5UYHaq8Gw9LPZA".to_string(),
            "ed25519:5E398aXyuB2rHmAgGSKVunaEFnvRDJA8v9WjBGv84sxXLSEHAphfo99xbRGmvghnx1befSyLNkiYVbu4M8aaSg8m".to_string(),
            "ed25519:3rZKJGN6qQDWqEKge3gFm4KqqmNWJ7B8VXSz9f5wEFjgwVU81U6nF4iFF75DvReKaqoRxncBTi5HL5n8UPx9n9g4".to_string(),
            "ed25519:3TYRq9LstrATmGetoT2daaK7LCuCtnoP6Vt6JfGe2GBT49iqQLGnj8g8AVDeUStvSbCjwVEhwYnvyCoAyrmGD1sp".to_string(),
            "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".to_string(),
        ]
    }

    fn dummy_ranks() -> Vec<MultiSigRank> {
        vec![
            MultiSigRank {
                min: 0,
                max_eq: 100,
                sig_num: 0,
            },
            MultiSigRank {
                min: 100,
                max_eq: 10000,
                sig_num: 1,
            },
            MultiSigRank {
                min: 10000,
                max_eq: 999999999999,
                sig_num: 2,
            },
        ]
    }

    #[tokio::test]
    async fn test_add_key_delete_key() {
        common::log::init_logger();
        let newcomer_pubkey = "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9";
        let main_account = "bcfffa8f19a9fe133510cf769702ad8bfdff4723f595c82c640ec048a225db4a";
        let master_pubkey = "bcfffa8f19a9fe133510cf769702ad8bfdff4723f595c82c640ec048a225db4a";
        let master_prikey = "331dde3ee69831fd2d8f0505a7f19b06c83bb14e11651debf29b8bf018e7d13ebcfffa8f19a9fe133510cf769702ad8bfdff4723f595c82c640ec048a225db4a";
        let client = ContractClient::<super::MultiSig>::new_update_cli()
            .await
            .unwrap();
        let master_list = client.get_master_pubkey_list(main_account).await.unwrap();

        //增加之前判断是否有
        if !master_list.contains(&newcomer_pubkey.to_string()) {
            let res = client.add_key(main_account, newcomer_pubkey).await.unwrap();
            let signature = common::encrypt::ed25519_sign_hex(master_prikey, &res.0).unwrap();
            let _test = crate::general::broadcast_tx_commit_from_raw2(&res.1, &signature).await;
        } else {
            debug!("newcomer_pubkey<{}> already is master", newcomer_pubkey);
        }

        //删除之前判断目标新公钥是否在，在的话就把新公钥之外的全删了
        let mut master_list = client.get_master_pubkey_list(main_account).await.unwrap();
        master_list.retain(|x| x != master_pubkey);
        if !master_list.is_empty() {
            //理论上生产环境不会出现3个master并存的场景
            for master_pubkey in master_list {
                let res = client
                    .delete_key(main_account, &master_pubkey)
                    .await
                    .unwrap();
                let signature = common::encrypt::ed25519_sign_hex(master_prikey, &res.0).unwrap();
                crate::general::broadcast_tx_commit_from_raw2(&res.1, &signature)
                    .await
                    .unwrap();
            }
        }
    }

    #[tokio::test]
    async fn test_get_master_keys() {
        let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await.unwrap();
        let account_id = "test".to_string();
        let res = multi_sig_cli.get_master_pubkey_list(&account_id).await;
        println!("{:?}", res);
    }

    pub fn gen_random_account_id() -> String {
        let relayer_name = &common::env::CONF.relayer_pool.account_id;
        let hex_str = generate_random_hex_string(8);
        let account_id = format!("{}.{}", hex_str, relayer_name);
        account_id
    }
}
