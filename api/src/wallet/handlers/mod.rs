use std::{ops::Deref, str::FromStr};

use blockchain::{coin::Coin, multi_sig::{MultiSig, MultiSigRank, StrategyData}, ContractClient};
use common::{
    data_structures::{account_manager::UserInfo, device_info::DeviceInfo, wallet::CoinType, KeyRole2},
    error_code::{BackendError, BackendRes, WalletError},
};
use models::{
    account_manager::{UserFilter, UserInfoView}, coin_transfer::{CoinTxFilter, CoinTxView}, device_info::{DeviceInfoFilter, DeviceInfoView}, PsqlOp
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::account_manager::user_info;
use common::error_code::BackendError::ChainError;


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
pub mod update_subaccount_hold_limit;
pub mod get_tx;
pub mod cancel_send_money;
pub mod gen_send_money;
pub mod get_need_sig_num;


pub fn get_uncompleted_tx(account: &str) -> Result<Vec<CoinTxView>> {
    CoinTxView::find(CoinTxFilter::BySenderUncompleted(account))
}


pub fn have_no_uncompleted_tx(account: &str) -> Result<(), BackendError> {
    let tx = get_uncompleted_tx(account)?;
    if !tx.is_empty() {
        Err(WalletError::HaveUncompleteTx)?;
    }
    Ok(())
}

pub fn get_freezn_amount(account: &str,coin:&CoinType) -> u128{
    let mut tx = get_uncompleted_tx(account).unwrap();
    tx.retain(|x| x.transaction.coin_type == *coin);
    tx.iter().map(|x| x.transaction.amount).sum()
}

pub async fn get_available_amount(account_id: &str,coin:&CoinType) -> BackendRes<u128>{
    let coin_cli = ContractClient::<Coin>::new(coin.clone()).map_err(|err| ChainError(err.to_string()))?;
    let balance = coin_cli.get_balance(account_id).await.unwrap().unwrap_or("0".to_string());
    let freezn_amount = get_freezn_amount(account_id, coin);
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


pub fn get_email(user_id:u32) -> Result<String,BackendError>{
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    Ok(user.user_info.email)
}


pub async fn get_servant_need(
    strategy: &Vec<MultiSigRank>,
    _coin: &CoinType,
    amount: u128,
) -> u8 {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let coin_price = 1;
    let transfer_value = amount * coin_price;
    strategy
        .iter()
        .find(|&rank| transfer_value >= rank.min && transfer_value < rank.max_eq)
        .map(|rank| rank.sig_num)
        .unwrap_or(0)    
}


pub fn get_role(strategy:&StrategyData,hold_key:Option<&str>) -> KeyRole2{
    if let Some(key) = hold_key {
        if strategy.master_pubkey == key {
            KeyRole2::Master
        }else if strategy.servant_pubkeys.contains(&key.to_string()) {
            KeyRole2::Servant
        }else{
            error!("unnormal device: key {} is not belong to current account",key);
            unreachable!("unnormal device");
        }
    }else {
        KeyRole2::Undefined
    }
}


//获取当前会话的用户信息、多签配置、设备信息的属性数据
pub async fn get_session_state(user_id:u32,device_id:&str) -> Result<(UserInfo,StrategyData,DeviceInfo)>{
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = &user.user_info.main_account;
    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let current_strategy = multi_sig_cli
        .get_strategy(main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.to_owned()))?;

    let mut device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(device_id, user_id))?;
    device.device_info.key_role = get_role(&current_strategy,device.device_info.hold_pubkey.as_deref());
    Ok((user.user_info,current_strategy,device.device_info))
}

pub fn check_role(current:KeyRole2,require:KeyRole2) -> Result<()>{
    if current != require {
        Err(WalletError::UneligiableRole(
            current,
            require,
        ))?;
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ServentSigDetail {
    pub pubkey: String,
    pub device_id:String,
    pub device_brand:String
}

impl FromStr for ServentSigDetail {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pubkey = s[..64].to_string();
        let _sig = s[64..].to_string();
        let device = DeviceInfoView::find_single(DeviceInfoFilter::ByHoldKey(&pubkey))?;
        Ok(Self{
            pubkey,
            device_id: device.device_info.id,
            device_brand: device.device_info.brand,
        })
    }
}