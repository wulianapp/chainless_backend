use blockchain::{coin::Coin, ContractClient};
use common::{
    data_structures::{wallet::CoinType, KeyRole2},
    error_code::{BackendError, BackendRes, WalletError},
};
use models::{
    account_manager::{UserFilter, UserInfoView}, coin_transfer::{CoinTxFilter, CoinTxView}, PsqlOp
};
use std::result::Result;

use crate::account_manager::user_info;

pub mod add_servant;
pub mod add_subaccount;
pub mod balance_list;
pub mod commit_newcomer_replace_master;
pub mod commit_servant_switch_master;
pub mod create_main_account;
pub mod device_list;
pub mod faucet_claim;
pub mod gen_newcomer_switch_master;
pub mod gen_servant_switch_master;
pub mod get_secret;
pub mod get_strategy;
pub mod pre_send_money;
pub mod pre_send_money_to_sub;
pub mod react_pre_send_money;
pub mod reconfirm_send_money;
pub mod remove_servant;
pub mod newcommer_switch_servant;
pub mod search_message;
pub mod servant_saved_secret;
pub mod update_security;
pub mod update_strategy;
pub mod upload_servant_sig;
pub mod sub_send_to_main;
pub mod tx_list;

pub fn get_uncompleted_tx(account: &str) -> Result<Vec<CoinTxView>, BackendError> {
    CoinTxView::find(CoinTxFilter::BySenderUncompleted(account))
}


pub fn have_no_uncompleted_tx(account: &str) -> Result<(), BackendError> {
    let tx = get_uncompleted_tx(&account)?;
    if !tx.is_empty() {
        Err(WalletError::HaveUncompleteTx)?;
    }
    Ok(())
}

pub fn get_freezn_amount(account: &str,coin:&CoinType) -> u128{
    let mut tx = get_uncompleted_tx(&account).unwrap();
    tx.retain(|x| x.transaction.coin_type == *coin);
    tx.iter().map(|x| x.transaction.amount).sum()
}

pub async fn get_available_amount(account_id: &str,coin:&CoinType) -> BackendRes<u128>{
    let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone());
    let balance = coin_cli.get_balance(&account_id).await.unwrap().unwrap_or("0".to_string());
    let freezn_amount = get_freezn_amount(&account_id, &coin);
    let total:u128 = balance.parse().unwrap();
    if total < freezn_amount{
        //todo:
        Err(WalletError::ExceedSubAccountHoldLimit)?
    }else {
        Ok(Some(total - freezn_amount))
    }
}

pub fn get_main_account(user_id:u32) -> Result<String,BackendError>{
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    Ok(user.user_info.main_account)
}
