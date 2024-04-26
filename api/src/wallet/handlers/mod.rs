use std::{num::ParseIntError, ops::Deref, str::FromStr, string::ParseError};

use anyhow::Result;
use blockchain::{
    coin::Coin,
    fees_call::FeesCall,
    multi_sig::{MultiSig, MultiSigRank, StrategyData},
    ContractClient,
};
use common::{
    data_structures::{account_manager::UserInfo, device_info::DeviceInfo, CoinType, KeyRole2},
    error_code::{BackendError, BackendRes, WalletError},
    utils::math::generate_random_hex_string,
};
use models::{
    account_manager::{UserFilter, UserInfoView},
    coin_transfer::{CoinTxFilter, CoinTxView},
    device_info::{DeviceInfoFilter, DeviceInfoView},
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{account_manager::user_info, utils::respond::BackendRespond};
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::*;
use common::error_code::{AccountManagerError,WalletError::*};


pub mod add_servant;
pub mod add_subaccount;
pub mod balance_list;
pub mod cancel_send_money;
pub mod commit_newcomer_replace_master;
pub mod commit_servant_switch_master;
pub mod create_main_account;
pub mod device_list;
pub mod faucet_claim;
pub mod gen_newcomer_switch_master;
pub mod gen_send_money;
pub mod gen_servant_switch_master;
pub mod get_fees_priority;
pub mod get_need_sig_num;
pub mod get_secret;
pub mod get_strategy;
pub mod get_tx;
pub mod newcommer_switch_servant;
pub mod pre_send_money;
pub mod pre_send_money_to_sub;
pub mod react_pre_send_money;
pub mod reconfirm_send_money;
pub mod remove_servant;
pub mod remove_subaccount;
pub mod search_message;
pub mod servant_saved_secret;
pub mod set_fees_priority;
pub mod sub_send_to_main;
pub mod tx_list;
pub mod update_security;
pub mod update_strategy;
pub mod update_subaccount_hold_limit;
pub mod upload_servant_sig;
pub mod estimate_transfer_fee;

const MIN_BASE_FEE:u128 = 1_000_000_000_000_000_000;

pub async fn gen_random_account_id(
    multi_sig_cli: &ContractClient<MultiSig>,
) -> Result<String, BackendError> {
    for _ in 0..10 {
        let relayer_name = &common::env::CONF.multi_sig_relayer_account_id;
        let hex_str = generate_random_hex_string(8);
        let account_id = format!("{}.{}", hex_str, relayer_name);
        //当前的以空master_key来判断是否账户存在
        let key = multi_sig_cli.get_master_pubkey_list(&account_id).await?;
        if key.is_empty() {
            return Ok(account_id);
        } else {
            warn!("account_id {} already register on chain", account_id);
        }
    }
    Err(BackendError::InternalError("".to_string()))
}

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

pub fn get_freezn_amount(account: &str, coin: &CoinType) -> u128 {
    let mut tx = get_uncompleted_tx(account).unwrap();
    tx.retain(|x| x.transaction.coin_type == *coin);
    tx.iter().map(|x| x.transaction.amount).sum()
}

pub async fn get_available_amount(account_id: &str, coin: &CoinType) -> BackendRes<u128> {
    let coin_cli =
        ContractClient::<Coin>::new(coin.clone()).map_err(|err| ChainError(err.to_string()))?;
    let balance = coin_cli
        .get_balance(account_id)
        .await
        .unwrap()
        .unwrap_or("0".to_string());
    let freezn_amount = get_freezn_amount(account_id, coin);
    let total: u128 = balance.parse().unwrap();
    if total < freezn_amount {
        Err(BackendError::InternalError(
            format!("{}(total) more than {}(freezn_amount)",total,freezn_amount)
        ))?
    } else {
        Ok(Some(total - freezn_amount))
    }
}

pub fn get_main_account(user_id: u32) -> Result<String, BackendError> {
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    if user.user_info.main_account.eq(""){
        Err(WalletError::NotSetSecurity)?
    }
    Ok(user.user_info.main_account)
}

pub fn get_email(user_id: u32) -> Result<String, BackendError> {
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    Ok(user.user_info.email)
}

//calculate total value for dollar
//目前的场景转账超过300兆才会溢出
//由于取整造成的精度丢失可以忽略
pub async fn get_value(coin: &CoinType, amount: u128) -> u128 {
    let fees_cli = ContractClient::<FeesCall>::new().unwrap();
    let (base_amount, quote_amount) = fees_cli.get_coin_price(coin).await.unwrap();
    amount * quote_amount / base_amount
}

pub async fn get_servant_need(strategy: &Vec<MultiSigRank>, coin: &CoinType, amount: u128) -> u8 {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let transfer_value = get_value(coin, amount).await;
    strategy
        .iter()
        .find(|&rank| transfer_value >= rank.min && transfer_value < rank.max_eq)
        .map(|rank| rank.sig_num)
        .unwrap_or(0)
}

pub fn get_role(strategy: &StrategyData, hold_key: Option<&str>) -> KeyRole2 {
    if let Some(key) = hold_key {
        if strategy.master_pubkey == key {
            KeyRole2::Master
        } else if strategy.servant_pubkeys.contains(&key.to_string()) {
            KeyRole2::Servant
        } else {
            /*** 
            //如果从设备被删之后，就变成了新设备
            error!(
                "unnormal device: key {} is not belong to current account",key
            );
            unreachable!("unnormal device");
            */
            KeyRole2::Undefined
        }
    } else {
        KeyRole2::Undefined
    }
}

//获取当前会话的用户信息、多签配置、设备信息的属性数据
//且已经进行过了多签
pub async fn get_session_state(
    user_id: u32,
    device_id: &str,
) -> Result<(UserInfo, StrategyData, DeviceInfo)> {
    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = &user.user_info.main_account;
    if user.user_info.main_account.eq(""){
        Err(WalletError::NotSetSecurity)?
    }
    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let current_strategy = multi_sig_cli
        .get_strategy(main_account)
        .await?
        .ok_or(BackendError::InternalError("main_account not found".to_string()))?;

    //注册过的一定有设备信息
    let mut device =
        DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(device_id, user_id))?;
    device.device_info.key_role =
        get_role(&current_strategy, device.device_info.hold_pubkey.as_deref());
    Ok((user.user_info, current_strategy, device.device_info))
}

pub fn check_role(current: KeyRole2, require: KeyRole2) -> Result<()> {
    if current != require {
        Err(WalletError::UneligiableRole(current, require))?;
    }
    Ok(())
}

pub async fn get_fees_priority(main_account:&str) -> BackendRes<Vec<CoinType>>{
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new()?;
    let fees_priority = fees_call_cli.get_fees_priority(&main_account).await?;
    Ok(Some(fees_priority))
}
//5u
//fixme: 查一次最多调用 1 + 5 * 2 的费用
pub async fn check_base_fee(main_account:&str) -> Result<(),BackendError> {
    let fee_coins = get_fees_priority(main_account)
    .await?
    .ok_or(InternalError("not set fees priority".to_string()))?;

    for coin  in fee_coins {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
        let balance = coin_cli.get_balance(main_account).await?
        .unwrap_or("0".to_string());
        let balance = balance.parse().map_err(|e:ParseIntError| e.to_string())?;
        let value = get_value(&coin, balance).await;
        if value > MIN_BASE_FEE{
            return Ok(());
        }
    }
    Err(WalletError::InsufficientAvailableBalance.into())
}

/***
pub async fn estimate_transfer_fee(
        main_account:&str,
        coin:&CoinType,
        amount: u128
    ) -> Result<(CoinType,u128,bool),BackendError> {
    let fee_coins = get_fees_priority(main_account).await?.ok_or(InternalError("not set fees priority".to_string()))?;
    let transfer_value = get_value(coin, amount).await;
    let fee_value = transfer_value * 999 / 1000 + MIN_BASE_FEE;
    //todo:
    let mut default_fee = (coin.to_owned(),0,false);

    for (index,coin)  in fee_coins.iter().enumerate() {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
        let balance = coin_cli.get_balance(main_account).await?
        .unwrap_or("0".to_string());
        let balance = balance.parse().map_err(|e:ParseIntError| e.to_string())?;
        let balance_value = get_value(&coin, balance).await;

        //fixme: 
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let (base_amount, quote_amount) = fees_cli.get_coin_price(coin).await.unwrap();

        if balance_value  > fee_value{
            return Ok((coin.to_owned(),fee_value * base_amount / quote_amount,true));
        }

        if index == 1 {
            default_fee = (coin.to_owned(),fee_value * base_amount / quote_amount,false)
        }
    }
    Ok(default_fee)
}
**/

// 1/1000
pub async fn check_protocal_fee(current: KeyRole2, require: KeyRole2) -> Result<()> {
    if current != require {
        Err(WalletError::UneligiableRole(current, require))?;
    }
    Ok(())
}

//base_fee + protocal_fee
pub fn check_fee(current: KeyRole2, require: KeyRole2) -> Result<()> {
    if current != require {
        Err(WalletError::UneligiableRole(current, require))?;
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ServentSigDetail {
    pub pubkey: String,
    pub device_id: String,
    pub device_brand: String,
}

impl FromStr for ServentSigDetail {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pubkey = s[..64].to_string();
        let _sig = s[64..].to_string();
        let device = DeviceInfoView::find_single(DeviceInfoFilter::ByHoldKey(&pubkey))?;
        Ok(Self {
            pubkey,
            device_id: device.device_info.id,
            device_brand: device.device_info.brand,
        })
    }
}
