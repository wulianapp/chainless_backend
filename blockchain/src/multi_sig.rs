use common::data_structures::wallet::get_support_coin_list;
use common::error_code::BackendError;
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;
//use ed25519_dalek::Signer;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use near_crypto::{ED25519SecretKey, PublicKey, SecretKey, Signer};
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use rand::rngs::OsRng;
use serde_json::json;

//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use lazy_static::lazy_static;
use near_crypto::InMemorySigner;
use near_primitives::account::Account;
use near_primitives::borsh::BorshSerialize;
use near_primitives::types::AccountId;
use near_crypto::Signature;

use common::data_structures::wallet::CoinType;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::coin::Coin;
use crate::general::get_access_key_list;
use crate::general::pubkey_from_hex_str;
use crate::general::{gen_transaction, safe_gen_transaction};
use crate::ContractClient;

pub struct MultiSig {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRank {
    pub min: u128,
    pub max_eq: u128,
    pub sig_num: u8,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SubAccConf {
    pub hold_value_limit: u128,
}

impl Default for MultiSigRank {
    fn default() -> Self {
        MultiSigRank {
            min: 0,
            //fixme: number out of range when u128::MAX
            max_eq: u64::MAX as u128,
            sig_num: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyData {
    pub master_pubkey: String,
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub servant_pubkeys: Vec<String>,
    pub sub_confs: HashMap<String,SubAccConf>,
}

impl ContractClient<MultiSig> {
    //fixme: gen once object
    pub fn new() -> Result<Self> {
        let prikey_str = &common::env::CONF.multi_sig_relayer_prikey;
        let contract = &common::env::CONF.multi_sig_contract;
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

    pub async fn get_total_value(&self, account_str: &str) -> Result<u128>{
        let coins = get_support_coin_list();
        let mut total_value = 0;
        for coin in coins {
            let coin_cli = ContractClient::<Coin>::new(coin)?;
            let balance = coin_cli.get_balance(account_str).await?;
            let balance:u128 = balance.unwrap_or("0".to_string()).parse()?;
            //todo: get price from contract
            let coin_price = 1;
            total_value +=  balance * coin_price;
        }
        Ok(total_value)
    }

    //fixeme:
    //用户永远只持有最后一个master的私钥
    //增加key的时候，新key永远不会放在末尾
    pub async fn get_master_pubkey(&self, account_str: &str) -> Result<String> {
        let list = get_access_key_list(account_str).await?.keys;
        if list.len() != 1 {
            error!("account have multi key {:?}", list);
            //panic!("todo");
        }
        //let key = list.first().unwrap().public_key.key_data();
        let key = list.last().unwrap().public_key.key_data();

        Ok(hex::encode(key))
    }

    pub async fn get_master_pubkey_list(&self, account_str: &str) -> Result<Vec<String>> {
        let list = get_access_key_list(account_str).await?.keys;
        let list  = list.iter()
            .map(|key| hex::encode(key.public_key.key_data()))
            .collect();
        Ok(list)
    }

    //add_master
    //这里先查询如果已经存在就不加了
    pub async fn add_key(&self, main_account: &str, new_key: &str) -> Result<(String, String)> {
        let master_pubkey = self.get_master_pubkey(main_account).await?;
        let master_pubkey = pubkey_from_hex_str(&master_pubkey)?;
        let main_account = AccountId::from_str(main_account).unwrap();
        self.gen_raw_with_caller(&main_account, &master_pubkey, "add_key", new_key)
            .await
    }

    //fixme：删除的这个理论上应该用新增加的key来签名，保证确实增加进去了，但是这样需要等待增加key执行完才行
    //为了减少前端的工作量，这里删除也用原有主私钥，也就是自己删自己
    //后期可以在链底层增加一个直接替换的接口
    pub async fn delete_key(
        &self,
        main_account: &str,
        delete_key: &str,
    ) -> Result<(String, String)> {
        let master_pubkey = self.get_master_pubkey(main_account).await?;
        let master_pubkey = pubkey_from_hex_str(&master_pubkey)?;
        let main_account = AccountId::from_str(main_account).unwrap();
        self.gen_raw_with_caller(&main_account, &master_pubkey, "delete_key", delete_key)
            .await
    }

    pub async fn get_strategy(&self, account_id: &str) -> Result<Option<StrategyData>> {
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({"user_account_id": user_account_id}).to_string();
        self.query_call("get_strategy", &args_str).await
    }

    pub async fn get_tx_state(&self, txs_index: Vec<u64>) -> Result<Option<Vec<(u64, bool)>>> {
        let args_str = json!({"txs_index": txs_index}).to_string();
        self.query_call("get_txs_state", &args_str).await
    }

    pub async fn init_strategy(
        &self,
        main_account_id: &str,
        subaccount_id: &str,
    ) -> Result<String> {
        //create account by send token
        let register_main_tx_id = self.register_account(main_account_id).await?;
        debug!("register_main_tx_id {}", register_main_tx_id);
        let register_tx_id = self.register_account(subaccount_id).await?;
        debug!("register_tx_id {}", register_tx_id);
        let sub_confs = HashMap::from([(subaccount_id,SubAccConf{ hold_value_limit: 100 })]);
        self.set_strategy(
            main_account_id,
            main_account_id,
            vec![],
            sub_confs,
            vec![MultiSigRank::default()],
        )
        .await
    }


    pub async fn init_strategy2(
        &self,
        main_account_id: &str,
        main_account_pubkey: &str,
        subaccount_id: &str,
        subaccount_pubkey: &str,
    ) -> Result<String> {
        //create account by send token
        let register_main_tx_id = self.register_account_with_name(
            main_account_id,
            main_account_pubkey
        ).await?;

        debug!("register_main_tx_id {}", register_main_tx_id);
        let register_tx_id = self.register_account_with_name(
            subaccount_id,
            subaccount_pubkey
        ).await?;

        debug!("register_tx_id {}", register_tx_id);
        let sub_confs = HashMap::from([(subaccount_id,SubAccConf{ hold_value_limit: 100 })]);
        self.set_strategy(
            main_account_id,
            main_account_pubkey,
            vec![],
            sub_confs,
            vec![MultiSigRank::default()],
        )
        .await
    }

    pub async fn remove_tx_index(&self, tx_index: u64) -> Result<String> {
        let args_str = json!({"index": tx_index}).to_string();
        self.commit_by_relayer("remove_tx_index", &args_str).await
    }

    pub async fn add_subaccount(&self, main_acc: &str, subacc: HashMap<&str,SubAccConf>) -> Result<String> {
        let main_acc = AccountId::from_str(main_acc)?;
        //let subacc = AccountId::from_str(subacc).unwrap();
        let sub_confs = subacc
        .into_iter()
        .map(|(acc_str,conf)| (AccountId::from_str(acc_str).unwrap(),conf))
        .collect::<HashMap<AccountId,SubAccConf>>();
        debug!("pre_add sub_confs {:?}",sub_confs);

        let args_str = json!({
            "main_account_id": main_acc,
            "new_sub": sub_confs
        })
        .to_string();
        self.commit_by_relayer("add_subaccounts", &args_str).await
    }

    pub async fn remove_subaccount(&self, main_acc: &str, subacc: &str) -> Result<String> {
        let main_acc = AccountId::from_str(main_acc)?;
        let subacc = AccountId::from_str(subacc)?;

        let args_str = json!({
            "main_account_id": main_acc,
            "accounts": vec![subacc]
        })
        .to_string();
        self.commit_by_relayer("remove_subaccounts", &args_str).await
    }

    pub async fn remove_account_strategy(&self, acc: String) -> Result<String> {
        let acc_id = AccountId::from_str(&acc).unwrap();
        let args_str = json!({"acc": acc_id}).to_string();
        self.commit_by_relayer("remove_account_strategy", &args_str)
            .await
    }

    pub async fn set_strategy(
        &self,
        account_id: &str,
        master_pubkey: &str,
        servant_pubkeys: Vec<String>,
        sub_confs: HashMap<&str,SubAccConf>,
        rank_arr: Vec<MultiSigRank>,
    ) -> Result<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id)?;
        let sub_confs = sub_confs
            .into_iter()
            .map(|(acc_str,conf)| (AccountId::from_str(acc_str).unwrap(),conf))
            .collect::<HashMap<AccountId,SubAccConf>>();
        let args_str = json!({
            "user_account_id": user_account_id,
            "master_pubkey": master_pubkey,
            "servant_pubkeys": servant_pubkeys,
            "sub_confs": sub_confs,
            "rank_arr": rank_arr
        })
        .to_string();
        self.commit_by_relayer("set_strategy2", &args_str).await
    }

    pub async fn update_rank(
        &self,
        account_id: &str,
        rank_arr: Vec<MultiSigRank>,
    ) -> Result<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id)?;
        let args_str = json!({
            "user_account_id": user_account_id,
            "rank_arr": rank_arr,
        })
        .to_string();
        self.commit_by_relayer("update_rank", &args_str).await
    }


    pub async fn update_subaccount_hold_limit(
        &self,
        main_account: &str,
        subaccount: &str,
        hold_limit: u128
    ) -> Result<String> {
        let main_account: AccountId = AccountId::from_str(main_account)?;
        let subaccount: AccountId = AccountId::from_str(subaccount)?;
        let args_str = json!({
            "user_account_id": main_account,
            "subaccount": subaccount,
            "hold_limit": hold_limit
        })
        .to_string();
        self.commit_by_relayer("update_subaccount_hold_limit", &args_str).await
    }

    async fn register_account(&self, user_id: &str) -> Result<String> {
        self.commit_by_relayer("register_account", user_id).await
    }

    async fn register_account_with_name(&self, 
        account_id: &str,
        pubkey:&str,
    ) -> Result<String> {
        let arg_str = format!("{}:{}",account_id,pubkey);
        self.commit_by_relayer("register_account_with_name", &arg_str).await
    }

    pub async fn update_servant_pubkey(
        &self,
        account_id: &str,
        servant_pubkey: Vec<String>,
    ) -> Result<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id)?;
        let args_str = json!({
            "user_account_id": user_account_id,
            "servant_device_pubkey": servant_pubkey,
        })
        .to_string();
        self.commit_by_relayer("update_servant_pubkey", &args_str)
            .await
    }

    pub async fn update_servant_pubkey_and_master(
        &self,
        account_id: &str,
        servant_pubkey: Vec<String>,
        master_pubkey: String,
    ) -> Result<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id)?;
        let args_str = json!({
            "user_account_id": user_account_id,
            "servant_device_pubkey": servant_pubkey,
            "master_pubkey": master_pubkey,
        })
        .to_string();
        self.commit_by_relayer("update_servant_pubkey", &args_str)
            .await
    }


    pub async fn update_master(
        &self,
        account_id: &str,
        master_pubkey: String,
    ) -> Result<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id)?;
        let args_str = json!({
            "user_account_id": user_account_id,
            "master_pubkey": master_pubkey,
        })
        .to_string();
        self.commit_by_relayer("update_master", &args_str)
            .await
    }

    //todo: 检查持仓限制
    pub async fn internal_transfer_main_to_sub(
        &self,
        master_sig: SignInfo,
        servant_sigs: Vec<SignInfo>,
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
        self.commit_by_relayer("internal_transfer_main_to_sub", &args_str)
            .await
    }

    pub async fn internal_transfer_sub_to_main(
        &self,
        main_account: &str,
        sub_sig: SignInfo,
        coin: CoinType,
        transfer_amount: u128,
    ) -> Result<String> {
        let main_account_id: AccountId = AccountId::from_str(main_account)?;
        let coin_tx = SubAccCoinTx {
            coin_id: coin.to_account_id(),
            amount: transfer_amount,
        };

        /*** 
         //todo: checkout and return error
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let public_key_bytes: Vec<u8> = hex::decode(sub_sig.pubkey).unwrap();
        let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
        let signature = ed25519_dalek::Signature::from_str(&sub_sig.signature).unwrap();
        println!("0002__coin_tx_str({}),signature({})",coin_tx_str,signature);
        public_key.verify_strict(coin_tx_str.as_bytes(), &signature).unwrap();
        ***/
        let args_str = json!({
            "main_account_id": main_account_id,
            "sub_sig": sub_sig,
            "coin_tx": coin_tx,
        })
        .to_string();

        self.commit_by_relayer("internal_transfer_sub_to_main", &args_str)
            .await
    }

    pub async fn gen_send_money_raw(
        &self,
        tx_index: u64,
        servant_device_sigs: Vec<SignInfo>,
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

        let args_str = json!({
            "tx_index": tx_index,
            "servant_device_sigs": servant_device_sigs,
            "coin_tx": coin_tx,
        })
        .to_string();
        self.gen_raw_with_caller(&caller_id, &caller_pubkey, "send_money", &args_str)
            .await
    }

    //for test
    pub fn ed25519_sign() {
        todo!()
    }

    pub async fn gen_send_money_raw_tx2(
        &self,
        tx_index: u64,
        _sender_account_id: &str,
        _sender_pubkey: &str,
        servant_device_sigs: Vec<SignInfo>,
        from: &str,
        to: &str,
        coin: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> Result<(String, String)> {
        let coin_tx = CoinTx {
            from: AccountId::from_str(from)?,
            to: AccountId::from_str(to)?,
            coin_id: coin.to_account_id(),
            amount: transfer_amount,
            expire_at,
            memo: None,
        };

        let args_str = json!({
            "tx_index": tx_index,
            "servant_device_sigs": servant_device_sigs,
            "coin_tx": coin_tx,
        })
        .to_string();

        self.gen_raw_with_relayer("send_money", &args_str).await
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
        let coin_tx_json = serde_json::to_string(&coin_tx_info).unwrap();
        let coin_tx_hex_str = hex::encode(coin_tx_json.as_bytes());
        Ok(coin_tx_hex_str)
    }


      //转账给跨链桥
       //todo: 弃用
    pub async fn internal_withdraw(
        &self,
        master_sig: SignInfo,
        servant_sigs: Vec<SignInfo>,
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
        self.commit_by_relayer("internal_withdraw", &args_str)
            .await
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignInfo {
    pub pubkey: String,
    pub signature: String,
}
impl FromStr for SignInfo{
    type Err = BackendError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //todo
        if s.len() < 64{
            Err(BackendError::RequestParamInvalid(s.to_string()))?;
        }
        Ok( SignInfo{
            pubkey: s[..64].to_string(),
            signature: s[64..].to_string(),
        })
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
        Ok(hex::encode(&near_sig_bytes))
    }else {
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
        Ok(hex::encode(&near_sig_bytes))
    }else {
        unreachable!("")
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::ed25519::signature::Signature;

    use near_crypto::{ED25519SecretKey, PublicKey};

    use super::*;
    use crate::general::broadcast_tx_commit_from_raw2;
    use crate::ContractClient;
    use common::utils::time::{now_millis, DAY1};

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
        let client = ContractClient::<super::MultiSig>::new().unwrap();
        let master_list = client.get_master_pubkey_list(main_account).await.unwrap();

        //增加之前判断是否有
        if !master_list.contains(&newcomer_pubkey.to_string()) {
            let res = client
                .add_key(main_account, newcomer_pubkey)
                .await
                .unwrap();
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
                crate::general::broadcast_tx_commit_from_raw2(&res.1, &signature).await;
            }
        }
    }

    /*** 
    #[tokio::test]
    async fn test_multi_sig_strategy() {
        let pri_key = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE";
        let secret_key: SecretKey = pri_key.parse().unwrap();
        let _secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let main_device_pubkey = get_pubkey(pri_key).unwrap();
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let _signer = near_crypto::InMemorySigner::from_secret_key(
            signer_account_id.to_owned(),
            secret_key.clone(),
        );

        let client = ContractClient::<super::MultiSig>::new().unwrap();
        let sender_id = AccountId::from_str(
            "6a7a4df96a60c\
        225f25394fd0195eb938eb1162de944d2c331dccef324372f45",
        )
        .unwrap();
        let _receiver_id = AccountId::from_str("test1").unwrap();

        let servant_pubkey = servant_keys().as_slice()[..2]
            .iter()
            .map(|x| {
                let secret_key = near_crypto::SecretKey::from_str(x).unwrap();
                let pubkey = secret_key.public_key().try_to_vec().unwrap();
                pubkey.as_slice()[1..].to_vec().encode_hex()
            })
            .collect::<Vec<String>>();

        println!("{:?}", servant_pubkey);

        let _ranks = dummy_ranks();
        let ranks = vec![MultiSigRank::default()];
        //let ranks = vec![];

        let strategy_str = client.get_strategy(&sender_id).await.unwrap();
        println!("strategy_str2 {:#?}", strategy_str);

        let set_strategy_res = client
            .set_strategy(&sender_id,&sender_id,servant_pubkey, HashMap::new(), ranks)
            .await
            .unwrap();
        println!("call set strategy txid {}", set_strategy_res);
    }
    #[tokio::test]
    async fn test_sig_near_ed25519() {
        let data_json = "{\"from\":\"6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45\",\
        \"to\":\"test1\",\"coin_id\":\"dw20.node0\",\"amount\":2,\"expire_at\":1706761873767,\"memo\":null}";
        let data: CoinTx = serde_json::from_str(data_json).unwrap();
        println!("{:#?}", data);

        let near_secret: SecretKey = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".parse().unwrap();

        let near_secret_bytes = near_secret.unwrap_as_ed25519().0;

        let ed25519_raw_bytes = near_secret_bytes;
        let data_str = serde_json::to_string(&data).unwrap();
        let data_bytes = data_str.as_bytes();

        //ed25519
        let secret_key = ed25519_dalek::Keypair::from_bytes(&ed25519_raw_bytes).unwrap();
        let sig = secret_key.sign(data_bytes);
        println!("ed25519_sig_res_bytes {:?}", sig.to_bytes());
        println!("ed25519_sig_res_hex {}", hex::encode(sig.to_bytes()));
        secret_key.verify(data_bytes, &sig).unwrap();

        //near
        let _near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(ed25519_raw_bytes));
        let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(near_secret_bytes));
        let _secret_key_bytes = near_secret.unwrap_as_ed25519().0.as_slice();
        let main_device_pubkey = get_pubkey(&near_secret.to_string()).unwrap();
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer = InMemorySigner::from_secret_key(signer_account_id, near_secret.clone());
        let signature = signer.sign(data_bytes);
        let near_sig_bytes = signature.try_to_vec().unwrap();
        let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
        println!("near_sig_res_bytes {:?}", ed25519_sig_bytes);
        println!("near_sig_res_hex {:?}", hex::encode(ed25519_sig_bytes));
    }
        **/

    /***
    #[tokio::test]
    async fn test_multi_sig_send_money2() {
        let pri_key = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE";
        let secret_key: SecretKey = pri_key.parse().unwrap();
        let _secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let main_device_pubkey = get_pubkey(&pri_key);
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(
            signer_account_id.to_owned(),
            secret_key.clone(),
        );

        let client = ContractClient::<super::MultiSig>::new();
        let sender_id = AccountId::from_str(
            "6a7a4df96a60c\
        225f25394fd0195eb938eb1162de944d2c331dccef324372f45",
        )
        .unwrap();
        let sender_pubkey = PublicKey::from_implicit_account(&sender_id).unwrap();

        let receiver_id = AccountId::from_str("test1").unwrap();

        let servant_pubkey = servant_keys().as_slice()[..2]
            .iter()
            .map(|x| {
                let secret_key = near_crypto::SecretKey::from_str(x).unwrap();
                let pubkey = secret_key.public_key().try_to_vec().unwrap();
                pubkey.as_slice()[1..].to_vec().encode_hex()
            })
            .collect::<Vec<String>>();

        println!("{:?}", servant_pubkey);

        let _ranks = dummy_ranks();

        let expire_at = now_millis() + DAY1;
        let coin_tx_hex_str = client
            .gen_send_money_info(
                sender_id.as_str(),
                receiver_id.as_str(),
                CoinType::DW20,
                3u128,
                expire_at,
            )
            .unwrap();

        println!("coin_tx_str {}", coin_tx_hex_str);

        //servant_device sign
        let servant_sigs: Vec<SignInfo> = servant_keys().as_slice()[..1]
            .iter()
            .map(|x| {
                let prikey: SecretKey = x.parse().unwrap();
                let prikey_byte = prikey.unwrap_as_ed25519().0;
                let data = hex::decode(&coin_tx_hex_str).unwrap();
                let signature = sign_data_by_near_wallet(prikey_byte, &data);

                SignInfo {
                    pubkey: get_pubkey(x),
                    signature,
                }
            })
            .collect();

        let sender_pubkey =
            hex::encode(sender_pubkey.try_to_vec().unwrap().as_slice()[1..].to_vec());
        let (txid, raw_str) = client
            .gen_send_money_raw_tx(
                sender_id.as_str(),
                &sender_pubkey.to_string(),
                servant_sigs,
                sender_id.as_str(),
                receiver_id.as_str(),
                CoinType::DW20,
                3u128,
                expire_at,
            )
            .await
            .unwrap();
        println!("send_money_txid {}", txid);

        //master_device sign
        let tx_hash = hex::decode(txid).unwrap();
        let signature = signer.sign(&tx_hash);
        let near_sig_bytes = signature.try_to_vec().unwrap();
        let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
        let sig_str = hex::encode(ed25519_sig_bytes);
        println!("sig_str: {} ", sig_str);

        //broadcast
        broadcast_tx_commit_from_raw2(&raw_str, &sig_str).await;
    }
    */
}
