use common::data_structures::coin_transaction::CoinTransaction;
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

use common::data_structures::CoinType;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::multi_sig::get_pubkey;
use crate::ContractClient;
use anyhow::Result;

lazy_static! {
    static ref MULTI_SIG_CID: AccountId = AccountId::from_str("multi_sig.node0").unwrap();
    static ref DW20_CID: AccountId = AccountId::from_str("dw20.node0").unwrap();
}

pub struct Coin {}

#[derive(Serialize, Deserialize, Debug)]
struct NRC20TransferArgs {
    receiver_id: AccountId,
    amount: u128,
    memo: Option<String>,
}

fn decode_action(acts: &Vec<Action>) -> Result<NRC20TransferArgs, String> {
    if acts.len() != 1 {
        Err("Only support one action")?;
    }
    if let FunctionCall(act) = acts.first().unwrap() {
        let FunctionCallAction {
            method_name,
            deposit,
            args: _,
            gas,
        } = act.deref();
        //todo: gas limit
        if method_name == "ft_transfer" && deposit.eq(&0u128) && gas <= &u64::MAX {
            if let Ok(args) = serde_json::from_slice::<NRC20TransferArgs>(&act.args) {
                Ok(args)
            } else {
                Err("func args decode failed")?
            }
        } else {
            Err("func call params is not eligible")?
        }
    } else {
        Err("Only support function call")?
    }
}

pub fn decode_coin_transfer(tx_raw: &str) -> Result<CoinTransaction, Box<dyn std::error::Error>> {
    let tx_hex = hex::decode(tx_raw)?;
    let transaction = Transaction::try_from_slice(&tx_hex)?;
    let (_hash, _) = transaction.get_hash_and_size();
    let _act = decode_action(&transaction.actions)?;
    Err("tmp".to_string())?
}

async fn get_balance(account: &AccountId) -> u128 {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: (*crate::coin::DW20_CID).clone(),
            method_name: "ft_balance_of".to_string(),
            args: FunctionArgs::from(
                json!({
                    "account_id":account.to_string()
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };
    let rep = crate::rpc_call(request).await.unwrap();

    if let QueryResponseKind::CallResult(result) = rep.kind {
        let amount_str: String = String::from_utf8(result.result)
            .unwrap()
            .split('\"')
            .collect();
        u128::from_str(&amount_str).unwrap()
    } else {
        unreachable!()
    }
}

impl ContractClient<Coin> {
    //fixme: gen once object
    pub fn new(coin: CoinType) -> Result<Self> {
        //multi_sig7_test
        //relayer_test=b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7
        let pri_key: SecretKey = "ed25519:3rSERwSqqyRNwSMaP61Kr3P96dQQGk4QwznTDNTxDMUqwTwkbBnjbwAjF39f98JSQzGXnzRWDUKb4HcpzDWyzWDc"
            .parse()
            .unwrap();
        let pubkey = get_pubkey(&pri_key.to_string())?;
        //bcfffa8f19a9fe133510cf769702ad8bfdff4723f595c82c640ec048a225db4a
        debug!("coin relayer punkey {}", pubkey);
        let account_id: AccountId = AccountId::from_str(&pubkey)?;
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);
        Ok(Self {
            deployed_at: coin.to_account_id(),
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

    pub async fn get_balance(&self, account_id: &str) -> Result<Option<String>> {
        let user_account_id = AccountId::from_str(account_id).unwrap();
        let args_str = json!({"account_id": user_account_id}).to_string();
        self.query_call("ft_balance_of", &args_str).await
    }
}

#[cfg(test)]
mod tests {
    use crate::general::gen_transaction;
    use common::data_structures::CoinType;
    use near_crypto::InMemorySigner;
    use near_primitives::borsh::{self, BorshSerialize};
    use near_primitives::types::AccountId;
    use serde_json::json;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    async fn gen_send_money_chain_tx_raw(
        from: &InMemorySigner,
        to: String,
        coin_type: CoinType,
        amount: u128,
        memo: Option<&str>,
    ) -> String {
        let transfer_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "ft_transfer".to_string(),
            args: json!({
                "receiver_id":  AccountId::from_str(&to).unwrap(),
                "amount": amount,
                "memo": memo,
            })
            .to_string()
            .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        }))];
        let mut transaction = gen_transaction(from, &coin_type.to_string())
            .await
            .unwrap();
        transaction.actions = transfer_actions;
        println!("{:?}", transaction);

        let raw_bytes = borsh::to_vec(&transaction.clone()).unwrap();
        hex::encode(&raw_bytes)
    }

    #[tokio::test]
    async fn test_call_coin_transfer() {
        let account_id = "1.node0".parse().unwrap();
        let pri_key = "ed25519:5VCjsh57P1hsSQzDoJyRKKMSLpMZjrgfVJvZWyjs6TgcdtbDnFgswGMtQ8MCcfnDQEkCUNUn7qEJ43TtC2Am5Xur".parse().unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);

        let raw_tx =
            gen_send_money_chain_tx_raw(&signer, "2.node0".to_string(), CoinType::CLY, 123, None)
                .await;
        println!("raw_tx {}", raw_tx);
        let decode_res = decode_coin_transfer(&raw_tx).unwrap();
        println!("decode_res {:?}", decode_res);
    }

    #[tokio::test]
    async fn test_call_coin_transfer_commit() {
        common::log::init_logger();
        let coin_cli = ContractClient::<Coin>::new(CoinType::DW20).unwrap();
        let receiver = "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb";
        let balance1 = coin_cli.get_balance(receiver).await.unwrap();
        println!("balance1 {}", balance1.unwrap());
        let _send_res = coin_cli.send_coin(receiver, 0u128).await.unwrap();
        // sleep(Duration::from_secs(3)).await;
        let balance2 = coin_cli.get_balance(receiver).await.unwrap();
        println!("balance2 {}", balance2.unwrap());
    }
}
