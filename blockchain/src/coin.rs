use near_primitives::borsh::BorshDeserialize;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest, SignedTransactionView};
use hex;
use near_primitives::transaction::Action::FunctionCall;
use serde_json::json;
use common::data_structures::wallet::{AddressConvert, CoinTransfer, CoinType, TransferStatus};
use common::utils::time::get_unix_time;
use log::warn;
use near_crypto::Signer;
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use serde::{Deserialize, Serialize};
use crate::ContractClient;
use crate::general::{broadcast_tx_commit, gen_transaction};


#[derive(Serialize, Deserialize, Debug)]
struct NRC20TransferArgs{
    receiver_id:String,
    amount:u128,
    memo:Option<String>
}

fn decode_action(acts: &Vec<Action>) -> Result<NRC20TransferArgs,String>{
    if acts.len() != 1 {
        Err("Only support one action")?;
    }
    if let FunctionCall(act) = acts.first().unwrap(){
        let FunctionCallAction{method_name,deposit,args,gas} = act;
        //todo: gas limit
        if method_name == "ft_transfer" &&  deposit.eq(&0u128) && gas <= &u64::MAX{
            if let Ok(args) = serde_json::from_slice::<NRC20TransferArgs>(&act.args){
                Ok(args)
            }else {
                Err("func args decode failed")?
            }
        }else {
            Err("func call params is not eligible")?
        }
    }else {
        Err("Only support function call")?
    }
}




pub fn decode_coin_transfer(tx_raw:&str) -> Result<CoinTransfer,String> {
    let tx_hex = hex::decode(tx_raw).unwrap();
    let transaction = Transaction::try_from_slice(&tx_hex).unwrap();
    let (hash,_) = transaction.get_hash_and_size();
    let act = decode_action(&transaction.actions)?;
    println!("{:?}",act);
    //todo: get coin type from contract addr

    Ok(CoinTransfer {
        tx_id: hash.to_string(),
        coin_type: CoinType::from_account_str(&transaction.receiver_id).unwrap(),
        from: transaction.signer_id.to_user_id(),
        to: act.receiver_id,
        amount: act.amount,
        status: TransferStatus::Pending,
        created_at: get_unix_time(),
        confirmed_at: 0,
    })
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use near_crypto::{InMemorySigner, SecretKey};
    use near_primitives::borsh::BorshSerialize;
    use near_primitives::types::{AccountId, Finality, FunctionArgs};
    use near_primitives::views::QueryRequest;
    use crate::{add, test1};
    use crate::general::broadcast_tx_commit_from_raw;
    use super::*;
    use serde_json::{from_slice};
    use serde::Deserialize;

    async fn gen_send_money_raw_data(from:&InMemorySigner,
                                         to:String,
                                         coin_type: CoinType,
                                         amount:u128,
                                         memo:Option<&str>) -> String{
        let transfer_actions = vec![Action::FunctionCall(*Box::new(FunctionCallAction {
            method_name: "ft_transfer".to_string(),
            args: json!({
                "receiver_id": to,
                "amount": amount,
                "memo": memo,
            })
                .to_string()
                .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        }))];
        let mut transaction = gen_transaction(from,&coin_type.to_account_str()).await;
        transaction.actions = transfer_actions;
        println!("{:?}",transaction);
        let raw_tx = transaction.try_to_vec().unwrap();
        hex::encode(raw_tx)
    }


    #[tokio::test]
    async fn test_call_coin_transfer(){
        let account_id = "cly.node0".parse().unwrap();
        let pri_key = "ed25519:5uZepzqRi74VFp1cP8L5RdUBQwhRvMoNxqXdLcguzm\
        uEiWSenww7drm1JQPY4mkiuUXFSnx5tu6Rb5cHLWG7AKSB".parse().unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id,pri_key);

        let raw_tx = gen_send_money_raw_data(&signer,
                                             "node0".to_string(),
                                             CoinType::CLY,
                                             123,
                                             None).await;
        println!("raw_tx {}",raw_tx);
        let decode_res = decode_coin_transfer(&raw_tx).unwrap();
        println!("decode_res {:?}",decode_res);
    }

}