use std::fmt::Debug;
use std::str::FromStr;
//use ed25519_dalek::Signer;
use ed25519_dalek::{Signature, Signer as DalekSigner};
use hex::ToHex;
use near_crypto::{ED25519SecretKey, SecretKey, Signer};
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, Transaction};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use serde_json::json;
use near_jsonrpc_client::{JsonRpcClient};
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use near_crypto::InMemorySigner;
use near_primitives::types::AccountId;
use lazy_static::lazy_static;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

use serde::{Deserialize, Serialize};
use common::data_structures::wallet::{AddressConvert, CoinType};
use common::utils::time::{DAY1, now_millis};
use crate::ContractClient;
use crate::general::{gen_transaction, safe_gen_transaction};

pub struct MultiSig {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRank {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

impl Default for MultiSigRank{
    fn default() -> Self {
        MultiSigRank {
            min: 0,
            max_eq: u128::MAX,
            sig_num: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyData {
    multi_sig_ranks: Vec<MultiSigRank>,
    main_device_pubkey: String,
    servant_device_pubkey: Vec<String>,
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
            deployed_at: "multi_sig.node0".parse().unwrap(),
            relayer: signer,
            phantom: Default::default(),
        }
    }

    async fn get_strategy(&self,user_account_id: &AccountId) -> Option<StrategyData> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: (self.deployed_at).clone(),
                method_name: "get_strategy".to_string(),
                args: FunctionArgs::from(json!({
                "user_account_id":user_account_id.to_string()
            }).to_string().into_bytes()),
            },
        };
        let rep = crate::general::call(request).await.unwrap();

        if let QueryResponseKind::CallResult(result) = rep.kind {
            let amount_str: String = String::from_utf8(result.result).unwrap();
            println!("strategy_str {}", amount_str);
            Some(serde_json::from_str::<StrategyData>(&amount_str).unwrap())
        } else {
            None
        }
    }

    async fn set_strategy(&self,user_account_id: &AccountId,
                          main_device_pubkey: String,
                          servant_pubkey: Vec<String>,
                          rank_arr: Vec<MultiSigRank>
    ) -> Result<String, String> {
        let set_strategy_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
            method_name: "set_strategy".to_string(),
            args: json!({
                "user_account_id": user_account_id,
                "main_device_pubkey": main_device_pubkey,
                "servant_device_pubkey": servant_pubkey,
                "rank_arr": rank_arr,
            })
                .to_string()
                .into_bytes(),
            gas: 300000000000000, // 100 TeraGas
            deposit: 0,
        }))];

        let mut transaction = gen_transaction(&self.relayer, &self.deployed_at.to_string()).await;
        transaction.actions = set_strategy_actions;

        //get from front
        let signature = self.relayer.sign(transaction.get_hash_and_size().0.as_ref());


        let tx = SignedTransaction::new(signature, transaction);
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };

        println!("call set strategy txid {}",&tx.get_hash().to_string());

        let rep = crate::general::call(request).await.unwrap();
        if let FinalExecutionStatus::Failure(error) = rep.status {
            Err(error.to_string())?;
        }
        let tx_id = rep.transaction.hash.to_string();
        Ok(tx_id)
    }

    pub fn gen_send_money_raw_tx_hex(){
        todo!()
    }

    //for test
    pub fn ed25519_sign(){
        todo!()
    }

    pub fn multi_sig_send_money(){
        todo!()
    }

    async fn send_money(
        &self,
        signer: InMemorySigner,
        servant_device_sigs: Vec<SignInfo>,
        coin_tx: CoinTx,
    ) -> Result<String, String>{
        //let CoinTx{from,to,coin_id,amount,memo} = coin_tx;
        let set_strategy_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
            method_name: "send_money".to_string(),
            args: json!({
                "servant_device_sigs": servant_device_sigs,
                "coin_tx": coin_tx,
            })
                .to_string()
                .into_bytes(),
            gas: 300000000000000, // 100 TeraGas
            deposit: 0,
        }))];

        let mut transaction = gen_transaction(&signer, &self.deployed_at.to_string()).await;
        transaction.actions = set_strategy_actions;


        //get by user
        let signature = signer.sign(transaction.get_hash_and_size().0.as_ref());

        let tx = SignedTransaction::new(signature, transaction);
        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        };

        println!("call set strategy txid {}",&tx.get_hash().to_string());

        let rep = crate::general::call(request).await.unwrap();
        if let FinalExecutionStatus::Failure(error) = rep.status {
            Err(error.to_string())?;
        }
        let tx_id = rep.transaction.hash.to_string();
        Ok(tx_id)
    }



    async fn gen_send_money_raw_tx(
        &self,
        sender_account_id:&str,
        sender_pubkey:&str,
        servant_device_sigs: Vec<SignInfo>,
        sender_id:&str,
        receiver_id:&str,
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at:u64
    ) -> Result<(String,String), String>{
        let coin_tx = CoinTx {
            from: AccountId::from_str(sender_id).unwrap(),
            to: AccountId::from_str(receiver_id).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
            amount: transfer_amount,
            expire_at,
            memo:None
        };

        let set_strategy_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
            method_name: "send_money".to_string(),
            args: json!({
                "servant_device_sigs": servant_device_sigs,
                "coin_tx": coin_tx,
            })
                .to_string()
                .into_bytes(),
            gas: 300000000000000, // 100 TeraGas
            deposit: 0,
        }))];

        //let mut transaction = gen_transaction(&signer, &self.deployed_at.to_string()).await;
        let mut transaction = safe_gen_transaction(sender_account_id,
                                                   sender_pubkey,
                                                   &self.deployed_at.to_string()).await;

        transaction.actions = set_strategy_actions;

        let hash =   transaction.get_hash_and_size().0.try_to_vec().unwrap();
        let txid = hex::encode(hash);
        let raw_bytes = transaction.try_to_vec().unwrap();
        let raw_str = hex::encode(raw_bytes);

        //let txid2 =   transaction.get_hash_and_size().0.to_string();
        //assert_eq!(txid1,txid2);
        Ok((txid,raw_str))
    }

    fn gen_send_money_info(
        &self,
        sender_id:&str,
        receiver_id:&str,
        coin_id: CoinType,
        transfer_amount: u128,
        expire_at: u64
    ) -> Result<String, String>{
        let coin_tx_info = CoinTx {
            from: AccountId::from_str(sender_id).unwrap(),
            to: AccountId::from_str(receiver_id).unwrap(),
            coin_id: AccountId::from_str(&coin_id.to_account_str()).unwrap(),
            amount: transfer_amount,
            expire_at,
            memo:None
        };
        let coin_tx_json = serde_json::to_string(&coin_tx_info).unwrap();
        let coin_tx_hex_str = hex::encode(coin_tx_json.as_bytes());
        Ok(coin_tx_hex_str)
    }
}




#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignInfo {
    pubkey: String,
    signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinTx {
    from: AccountId,
    to: AccountId,
    coin_id:AccountId,
    amount:u128,
    expire_at: u64,
    memo:Option<String>
}

lazy_static! {
    static ref MULTI_SIG_CID: AccountId = AccountId::from_str("multi_sig.node0").unwrap();
}

fn get_pubkey(pri_key_str:&str) -> String{
    let secret_key = near_crypto::SecretKey::from_str(pri_key_str).unwrap();
    let pubkey = secret_key.public_key().try_to_vec().unwrap();
    pubkey.as_slice()[1..].to_vec().encode_hex()
}

fn ed25519_sign_data(prikey_bytes:&[u8], data:&[u8]) -> String{
    println!("ed25519_secret {:?}",prikey_bytes);
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    let sig = secret_key.sign(data);
    sig.to_string()
}

fn sign_data_by_near_wallet(prikey_bytes: [u8;64], data:&[u8]) -> String{
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
    use hex::encode;
    use near_crypto::{ED25519SecretKey, PublicKey};
    use near_primitives::state_record::StateRecord::Contract;
    use common::utils::time::{DAY1, now_millis, now_nanos};
    use crate::ContractClient;
    use crate::general::{broadcast_tx_commit_from_raw, broadcast_tx_commit_from_raw2};
    use super::*;

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
    async fn test_multi_sig_send_money() {
        let pri_key = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE";
        let secret_key: SecretKey = pri_key.parse().unwrap();
        let secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let main_device_pubkey = get_pubkey(&pri_key);
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id.to_owned(), secret_key.clone());

        let client = ContractClient::<super::MultiSig>::new();
        let sender_id = AccountId::from_str("6a7a4df96a60c\
        225f25394fd0195eb938eb1162de944d2c331dccef324372f45").unwrap();
        let receiver_id = AccountId::from_str("test1").unwrap();

        let servant_pubkey = servant_keys().as_slice()[..2].iter().map(|x| {
            let secret_key = near_crypto::SecretKey::from_str(x).unwrap();
            let pubkey = secret_key.public_key().try_to_vec().unwrap();
            pubkey.as_slice()[1..].to_vec().encode_hex()
        }).collect::<Vec<String>>();

        println!("{:?}",servant_pubkey);

        let ranks = dummy_ranks();

        //let strategy_str = client.get_strategy(&sender_id).await;
        //println!("strategy_str2 {:#?}", strategy_str);

        //let set_strategy_res = client.set_strategy(&sender_id,sender_id.to_string(),servant_pubkey,ranks).await.unwrap();
        //println!("call set strategy txid {}",set_strategy_res);
        //send_money 1 dw20, 1 servant is enough


        let transfer_amount = 2;
        let coin_tx_info = CoinTx {
            from: sender_id,
            to: receiver_id,
            coin_id: AccountId::from_str("dw20.node0").unwrap(),
            amount: transfer_amount,
            expire_at: now_millis() + DAY1,
            memo:None
        };
        let coin_tx_json = serde_json::to_string(&coin_tx_info).unwrap();
        println!("coin_tx_json {}",coin_tx_json);
        let coin_tx_hex_str = hex::encode(coin_tx_json.as_bytes());
        println!("coin_tx_str {}",coin_tx_hex_str);


        let sigs: Vec<SignInfo> = servant_keys().as_slice()[..1].iter().map(|x|
            {
                let prikey: SecretKey = x.parse().unwrap();
                let prikey_byte = prikey.unwrap_as_ed25519().0;
                let data = coin_tx_json.as_bytes();
                let signature = sign_data_by_near_wallet(prikey_byte,data);

                SignInfo {
                    pubkey: get_pubkey(x),
                    //signature: ed25519_sign_data(prikey_byte, &coin_tx_json.as_bytes()),
                    signature,
                }
            }
        ).collect();
        let send_money_txid = client.send_money(signer, sigs, coin_tx_info).await.unwrap();
        println!("send_money_txid {}", send_money_txid);
    }

    #[tokio::test]
    async fn test_sig_near_ed25519(){
        let data_json = "{\"from\":\"6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45\",\
        \"to\":\"test1\",\"coin_id\":\"dw20.node0\",\"amount\":2,\"expire_at\":1706761873767,\"memo\":null}";
        let data : CoinTx = serde_json::from_str(data_json).unwrap();
        println!("{:#?}",data);

        let near_secret: SecretKey = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".parse().unwrap();

        let near_secret_bytes = near_secret.unwrap_as_ed25519().0;
        let ed25519_raw_bytes = near_secret_bytes.clone();
        let data_str = serde_json::to_string(&data).unwrap();
        let data_bytes= data_str.as_bytes();

        //ed25519
        let secret_key = ed25519_dalek::Keypair::from_bytes(&ed25519_raw_bytes).unwrap();
        let sig = secret_key.sign(data_bytes);
        println!("ed25519_sig_res_bytes {:?}",sig.to_bytes());
        println!("ed25519_sig_res_hex {}",hex::encode(sig.to_bytes()));
        let test2 = secret_key.verify(data_bytes,&sig).unwrap();

        //near
        let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(ed25519_raw_bytes));
        let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(near_secret_bytes));
        let secret_key_bytes = near_secret.unwrap_as_ed25519().0.as_slice();
        let main_device_pubkey = get_pubkey(&near_secret.to_string());
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer = InMemorySigner::from_secret_key(signer_account_id.to_owned(), near_secret.clone());
        let signature = signer.sign(&data_bytes);
        let near_sig_bytes = signature.try_to_vec().unwrap();
        let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
        println!("near_sig_res_bytes {:?}",ed25519_sig_bytes);
        println!("near_sig_res_hex {:?}",hex::encode(ed25519_sig_bytes));

    }


    #[tokio::test]
    async fn test_multi_sig_send_money2() {
        let pri_key = "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE";
        let secret_key: SecretKey = pri_key.parse().unwrap();
        let secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
        //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
        let main_device_pubkey = get_pubkey(&pri_key);
        let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id.to_owned(), secret_key.clone());

        let client = ContractClient::<super::MultiSig>::new();
        let sender_id = AccountId::from_str("6a7a4df96a60c\
        225f25394fd0195eb938eb1162de944d2c331dccef324372f45").unwrap();
        let sender_pubkey = PublicKey::from_implicit_account(&sender_id).unwrap();



        let receiver_id = AccountId::from_str("test1").unwrap();

        let servant_pubkey = servant_keys().as_slice()[..2].iter().map(|x| {
            let secret_key = near_crypto::SecretKey::from_str(x).unwrap();
            let pubkey = secret_key.public_key().try_to_vec().unwrap();
            pubkey.as_slice()[1..].to_vec().encode_hex()
        }).collect::<Vec<String>>();

        println!("{:?}",servant_pubkey);

        let ranks = dummy_ranks();

        let expire_at = now_millis() + DAY1;
        let coin_tx_hex_str = client.gen_send_money_info(
            sender_id.as_str(),
            receiver_id.as_str(),
            CoinType::DW20,
            3u128,
            expire_at
        ).unwrap();

        println!("coin_tx_str {}",coin_tx_hex_str);


        //servant_device sign
        let servant_sigs: Vec<SignInfo> = servant_keys().as_slice()[..1].iter().map(|x|
            {
                let prikey: SecretKey = x.parse().unwrap();
                let prikey_byte = prikey.unwrap_as_ed25519().0;
                let data = hex::decode(&coin_tx_hex_str).unwrap();
                let signature = sign_data_by_near_wallet(prikey_byte,&data);

                SignInfo {
                    pubkey: get_pubkey(x),
                    signature,
                }
            }
        ).collect();

        let (txid,raw_str) = client.gen_send_money_raw_tx(
                                                    sender_id.as_str(),
                                                    &sender_pubkey.to_string(),
                                                    servant_sigs,
                                                    sender_id.as_str(),
                                                    receiver_id.as_str(),
                                                    CoinType::DW20,
                                                    3u128,
                                                    expire_at
        ).await.unwrap();
        println!("send_money_txid {}", txid);

        //master_device sign
        let tx_hash = hex::decode(txid).unwrap();
        let signature = signer.sign(&tx_hash);
        let sig_str = hex::encode(signature.try_to_vec().unwrap());
        println!("sig_str: {} ",sig_str);

        //broadcast
        broadcast_tx_commit_from_raw2(&raw_str,&sig_str).await;

    }
}


