use common::{
    data_structures::KeyRole2,
    error_code::{BackendError, BackendRes, WalletError},
};
use models::{
    coin_transfer::{CoinTxFilter, CoinTxView},
    PsqlOp,
};
use std::result::Result;

pub mod add_servant;
pub mod add_subaccount;
pub mod balance_list;
pub mod commit_newcomer_replace_master;
pub mod commit_servant_switch_master;
pub mod create_main_account;
pub mod device_list;
pub mod direct_send_money;
pub mod faucet_claim;
pub mod gen_newcomer_replace_master;
pub mod gen_servant_switch_master;
pub mod get_device_secret;
pub mod get_master_secret;
pub mod get_secret;
pub mod get_strategy;
pub mod pre_send_money;
pub mod react_pre_send_money;
pub mod reconfirm_send_money;
pub mod remove_servant;
pub mod replace_servant;
pub mod search_message;
pub mod send_money;
pub mod servant_replace_master;
pub mod servant_saved_secret;
pub mod update_security;
pub mod update_strategy;
pub mod upload_servant_sig;

pub fn have_no_uncompleted_tx(account: &str) -> Result<(), BackendError> {
    let tx = CoinTxView::find(CoinTxFilter::BySenderUncompleted(account))?;
    if !tx.is_empty() {
        Err(WalletError::HaveUncompleteTx)?;
    }
    Ok(())
}
