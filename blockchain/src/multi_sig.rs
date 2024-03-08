use std::fmt::Debug;
use std::str::FromStr;
use common::error_code::BackendError;
use common::error_code::BackendRes;
//use ed25519_dalek::Signer;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use near_crypto::{ED25519SecretKey, PublicKey, SecretKey, Signer};
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use serde_json::json;

//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use lazy_static::lazy_static;
use near_crypto::InMemorySigner;
use near_primitives::account::Account;
use near_primitives::borsh::BorshSerialize;
use near_primitives::types::AccountId;

use common::data_structures::wallet::{AddressConvert, CoinType};
use serde::{Deserialize, Serialize};

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
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: Vec<AccountId>,
}



impl ContractClient<MultiSig> {
    //fixme: gen once object
    pub fn new() -> Self {
        let pri_key: SecretKey = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE"
            .parse()
            .unwrap();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let pubkey = get_pubkey(&pri_key.to_string());
        let account_id = AccountId::from_str(&pubkey).unwrap();

        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);
        Self {
            deployed_at: "multi_sig6.node0".parse().unwrap(),
            relayer: signer,
            phantom: Default::default(),
        }
    }

    pub async fn get_master_pubkey(&self,account_str: &str) -> String{
        let list = get_access_key_list(account_str).await.keys;
        if list.len() != 1 {
            panic!("todo");
        } 
        let key = list.first().unwrap().public_key.key_data();
        hex::encode(key)
    }

    pub async fn get_strategy(&self, account_id: &str) -> BackendRes<StrategyData> {
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({"user_account_id": user_account_id}).to_string();
        self.query_call("get_strategy", &args_str).await
    }
     

    pub async fn get_tx_state(&self, txs_index:Vec<u64>) -> BackendRes<Vec<(u64,bool)>> {
        let args_str = json!({"txs_index": txs_index}).to_string();
        self.query_call("get_txs_state", &args_str).await
    }


    pub async fn init_strategy(
        &self,
        main_account_id: &str,
        subaccount_id: &str,
    ) -> BackendRes<String> {
        self.set_strategy(
            main_account_id,
            vec![],
                            vec![subaccount_id.to_owned()],
            vec![MultiSigRank::default()],
        )
        .await
    }

    pub async fn remove_tx_index(
        &self,
        tx_index:u64
    ) -> BackendRes<String> {
        let args_str = json!({"index": tx_index}).to_string();
        self.commit_by_relayer("remove_tx_index", &args_str).await
    }

    pub async fn add_subaccount(
        &self,
        main_acc:&str,
        subacc:&str,
    ) -> BackendRes<String> {
        let main_acc = AccountId::from_str(&main_acc).unwrap();
        let subacc = AccountId::from_str(&subacc).unwrap();

        let args_str = json!({
            "main_account_id": main_acc,
            "accounts": vec![subacc]
        }).to_string();
        self.commit_by_relayer("add_subaccounts", &args_str).await
    }

    pub async fn remove_subaccount(
        &self,
        main_acc:&str,
        subacc:&str,
        ) -> BackendRes<String> {
        let main_acc = AccountId::from_str(&main_acc).unwrap();
        let subacc = AccountId::from_str(&subacc).unwrap();

        let args_str = json!({
            "main_account_id": main_acc,
            "accounts": vec![subacc]
        }).to_string();
        self.commit_by_relayer("remove_tx_index", &args_str).await
    }


    pub async fn remove_account_strategy(
        &self,
        acc:String
    ) -> BackendRes<String> {
        let acc_id = AccountId::from_str(&acc).unwrap();
        let args_str = json!({"acc": acc_id}).to_string();
        self.commit_by_relayer("remove_account_strategy", &args_str).await
    }

    pub async fn set_strategy(
        &self,
        account_id: &str,
        servant_pubkeys: Vec<String>,
        subaccounts: Vec<String>,
        rank_arr: Vec<MultiSigRank>,
    ) -> BackendRes<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id).unwrap();
        let subaccounts = subaccounts
        .iter()
        .map(|acc_str| AccountId::from_str(acc_str).unwrap())
        .collect::<Vec<AccountId>>();
        let args_str = json!({
            "user_account_id": user_account_id,
            "servant_pubkeys": servant_pubkeys,
            "subaccounts": subaccounts,
            "rank_arr": rank_arr
        })
        .to_string();
        self.commit_by_relayer("set_strategy2", &args_str).await
    }

    pub async fn update_rank(
        &self,
        account_id: &str,
        rank_arr: Vec<MultiSigRank>,
    ) -> BackendRes<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "user_account_id": user_account_id,
            "rank_arr": rank_arr,
        })
        .to_string();
        self.commit_by_relayer("update_rank", &args_str).await
    }

    pub async fn update_servant_pubkey(
        &self,
        account_id: &str,
        servant_pubkey: Vec<String>,
    ) -> BackendRes<String> {
        let user_account_id: AccountId = AccountId::from_str(account_id).unwrap();
        let args_str = json!({
            "user_account_id": user_account_id,
            "servant_device_pubkey": servant_pubkey,
        })
        .to_string();
        self.commit_by_relayer("update_servant_pubkey", &args_str).await
    }


    pub async fn internal_transfer_main_to_sub(
        &self,
        master_sig: SignInfo,
        servant_sigs: Vec<SignInfo>,
        from: &str,
        to: &str,
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> BackendRes<String> {


        let coin_tx = CoinTx {
            from: AccountId::from_str(from).unwrap(),
            to: AccountId::from_str(to).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
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
        self.commit_by_relayer("internal_transfer_main_to_sub", &args_str).await
    }

    
    pub async fn internal_transfer_sub_to_main(
        &self,
        main_account: &str,
        servant_sig: SignInfo,
        coin_id: CoinType,
        transfer_amount: u128,
    ) -> BackendRes<String> {
        let main_account_id: AccountId = AccountId::from_str(main_account).unwrap();
        let coin_tx = SubAccCoinTx {
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
            amount: transfer_amount,                    
        };

        let args_str = json!({
            "main_account_id": main_account_id,
            "sub_sig": servant_sig,
            "coin_tx": coin_tx,
        })
        .to_string();
        self.commit_by_relayer("internal_transfer_main_to_sub", &args_str).await
    }



    pub async fn gen_send_money_raw(
        &self,
        tx_index: u64,
        servant_device_sigs: Vec<SignInfo>,
        from: &str,
        to: &str,
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> BackendRes<(String, String)> {
        let caller_id = AccountId::from_str(from).unwrap();
        let caller_pubkey_str = self.get_master_pubkey(from).await;
        let caller_pubkey = pubkey_from_hex_str(&caller_pubkey_str);

        let coin_tx = CoinTx {
            from: AccountId::from_str(from).unwrap(),
            to: AccountId::from_str(to).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
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
        self.gen_raw_with_caller(&caller_id, &caller_pubkey, "send_money", &args_str).await
    }

    //for test
    pub fn ed25519_sign() {
        todo!()
    }


    pub async fn gen_send_money_raw_tx2(
        &self,
        tx_index: u64,
        sender_account_id: &str,
        sender_pubkey: &str,
        servant_device_sigs: Vec<SignInfo>,
        from: &str,
        to: &str,
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> BackendRes<(String, String)> {
        let coin_tx = CoinTx {
            from: AccountId::from_str(from).unwrap(),
            to: AccountId::from_str(to).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
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
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at: u64,
    ) -> Result<String, String> {
        let coin_tx_info = CoinTx {
            from: AccountId::from_str(sender_id).unwrap(),
            to: AccountId::from_str(receiver_id).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
            amount: transfer_amount,
            expire_at,
            memo: None,
        };
        let coin_tx_json = serde_json::to_string(&coin_tx_info).unwrap();
        let coin_tx_hex_str = hex::encode(coin_tx_json.as_bytes());
        Ok(coin_tx_hex_str)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignInfo {
    pub pubkey: String,
    pub signature: String,
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
pub struct SubAccCoinTx {
    coin_id:AccountId,
    amount:u128,
}

lazy_static! {
    static ref MULTI_SIG_CID: AccountId = AccountId::from_str("multi_sig.node0").unwrap();
}

fn get_pubkey(pri_key_str: &str) -> String {
    let secret_key = near_crypto::SecretKey::from_str(pri_key_str).unwrap();
    let pubkey = secret_key.public_key().try_to_vec().unwrap();
    pubkey.as_slice()[1..].to_vec().encode_hex()
}

fn pubkey_from_hex(hex_str: &str) -> PublicKey {
    println!("pubkey_from_hex {}", hex_str);
    let sender_id = AccountId::from_str(hex_str).unwrap();
    let sender_pubkey = PublicKey::from_implicit_account(&sender_id).unwrap();
    sender_pubkey
}

fn ed25519_sign_data(prikey_bytes: &[u8], data: &[u8]) -> String {
    println!("ed25519_secret {:?}", prikey_bytes);
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    let sig = secret_key.sign(data);
    sig.to_string()
}

pub fn ed25519_sign_data2(prikey_bytes_hex: &str, data_hex: &str) -> String {
   let prikey_bytes = hex::decode(prikey_bytes_hex).unwrap();
    let data = hex::decode(data_hex).unwrap();

    println!("ed25519_secret {:?}", prikey_bytes);
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    let sig = secret_key.sign(&data);
    sig.to_string()
}

pub fn sign_data_by_near_wallet2(prikey_str: &str, data_str: &str) -> String {
    let prikey: SecretKey = prikey_str.parse().unwrap();
    let prikey_bytes = prikey.unwrap_as_ed25519().0;
    let data = hex::decode(data_str).unwrap();

    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string());
    let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
    let signer = InMemorySigner::from_secret_key(signer_account_id.to_owned(), near_secret.clone());
    let signature = signer.sign(&data);
    let near_sig_bytes = signature.try_to_vec().unwrap();
    let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
    hex::encode(&ed25519_sig_bytes)
}

pub fn sign_data_by_near_wallet(prikey_bytes: [u8; 64], data: &[u8]) -> String {
    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string());
    let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
    let signer = InMemorySigner::from_secret_key(signer_account_id.to_owned(), near_secret.clone());
    let signature = signer.sign(data);
    let near_sig_bytes = signature.try_to_vec().unwrap();
    let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
    hex::encode(&ed25519_sig_bytes)
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
    async fn test_multi_sig_strategy() {
        let pri_key = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE";
        let secret_key: SecretKey = pri_key.parse().unwrap();
        let _secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let main_device_pubkey = get_pubkey(&pri_key);
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let _signer = near_crypto::InMemorySigner::from_secret_key(
            signer_account_id.to_owned(),
            secret_key.clone(),
        );

        let client = ContractClient::<super::MultiSig>::new();
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

        let strategy_str = client.get_strategy(&sender_id).await;
        println!("strategy_str2 {:#?}", strategy_str);

        let set_strategy_res = client
            .set_strategy(&sender_id, servant_pubkey, vec![],ranks)
            .await
            .unwrap();
        println!("call set strategy txid {}", set_strategy_res.unwrap());
    }

    #[tokio::test]
    async fn test_sig_near_ed25519() {
        let data_json = "{\"from\":\"6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45\",\
        \"to\":\"test1\",\"coin_id\":\"dw20.node0\",\"amount\":2,\"expire_at\":1706761873767,\"memo\":null}";
        let data: CoinTx = serde_json::from_str(data_json).unwrap();
        println!("{:#?}", data);

        let near_secret: SecretKey = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".parse().unwrap();

        let near_secret_bytes = near_secret.unwrap_as_ed25519().0;

        let ed25519_raw_bytes = near_secret_bytes.clone();
        let data_str = serde_json::to_string(&data).unwrap();
        let data_bytes = data_str.as_bytes();

        //ed25519
        let secret_key = ed25519_dalek::Keypair::from_bytes(&ed25519_raw_bytes).unwrap();
        let sig = secret_key.sign(data_bytes);
        println!("ed25519_sig_res_bytes {:?}", sig.to_bytes());
        println!("ed25519_sig_res_hex {}", hex::encode(sig.to_bytes()));
        let _test2 = secret_key.verify(data_bytes, &sig).unwrap();

        //near
        let _near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(ed25519_raw_bytes));
        let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(near_secret_bytes));
        let _secret_key_bytes = near_secret.unwrap_as_ed25519().0.as_slice();
        let main_device_pubkey = get_pubkey(&near_secret.to_string());
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer =
            InMemorySigner::from_secret_key(signer_account_id.to_owned(), near_secret.clone());
        let signature = signer.sign(&data_bytes);
        let near_sig_bytes = signature.try_to_vec().unwrap();
        let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
        println!("near_sig_res_bytes {:?}", ed25519_sig_bytes);
        println!("near_sig_res_hex {:?}", hex::encode(ed25519_sig_bytes));
    }

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
