use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::AccountId;

use hex;
use near_primitives::transaction::Action::FunctionCall;

use common::data_structures::wallet::{AddressConvert, CoinTransaction, CoinTxStatus, CoinType};

use serde::{Deserialize, Serialize};

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
        } = act;
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
    let (hash, _) = transaction.get_hash_and_size();
    let act = decode_action(&transaction.actions)?;

    Ok(CoinTransaction {
        tx_id: hash.to_string(),
        //receiver_id is contract account of coin
        coin_type: CoinType::from_account_str(&transaction.receiver_id.to_string())?,
        sender: transaction.signer_id.to_user_id(),
        receiver: act.receiver_id.to_user_id(),
        amount: act.amount,
        //fixme: deal with status
        status: CoinTxStatus::Created,
        raw_data: tx_raw.to_string(),
        signatures: vec![],
    })
}

#[cfg(test)]
mod tests {
    use near_crypto::InMemorySigner;
    use near_primitives::borsh::BorshSerialize;
    use near_primitives::types::AccountId;
    use std::str::FromStr;

    use super::*;

    async fn gen_send_money_raw_data(
        from: &InMemorySigner,
        to: String,
        coin_type: CoinType,
        amount: u128,
        memo: Option<&str>,
    ) -> String {
        let transfer_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
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
        let mut transaction = gen_transaction(from, &coin_type.to_account_str()).await;
        transaction.actions = transfer_actions;
        println!("{:?}", transaction);
        let raw_tx = transaction.try_to_vec().unwrap();
        hex::encode(raw_tx)
    }

    #[tokio::test]
    async fn test_call_coin_transfer() {
        let account_id = "1.node0".parse().unwrap();
        let pri_key = "ed25519:5VCjsh57P1hsSQzDoJyRKKMSLpMZjrgfVJvZWyjs6TgcdtbDnFgswGMtQ8MCcfnDQEkCUNUn7qEJ43TtC2Am5Xur".parse().unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);

        let raw_tx =
            gen_send_money_raw_data(&signer, "2.node0".to_string(), CoinType::CLY, 123, None).await;
        println!("raw_tx {}", raw_tx);
        let decode_res = decode_coin_transfer(&raw_tx).unwrap();
        println!("decode_res {:?}", decode_res);
    }
}
