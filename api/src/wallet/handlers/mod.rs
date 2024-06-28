use std::num::ParseIntError;

use anyhow::Result;
use blockchain::{
    coin::Coin,
    multi_sig::{MultiSig, MultiSigRank, StrategyData},
    ContractClient,
};
use common::{
    data_structures::{
        account_manager::UserInfo, coin_transaction::CoinSendStage, device_info::DeviceInfo,
    },
    utils::{
        math::{coin_amount::raw2display, generate_random_hex_string},
        time::now_millis,
    },
};
use models::{
    account_manager::{UserFilter, UserInfoEntity},
    coin_transfer::{CoinTxEntity, CoinTxFilter},
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::*;

pub use common::prelude::*;
use common::utils::math::*;

pub mod cancel_send_money;
pub mod device_list;
pub mod get_secret;
pub mod get_tx;
pub mod pre_send_money;
pub mod react_pre_send_money;
pub mod reconfirm_send_money;
pub mod search_message;
pub mod servant_saved_secret;
pub mod tx_list;
pub mod update_security;
pub mod upload_servant_sig;

//短地址允许碰撞的次数
pub const FIND_VALID_ACCOUNT_RETRY_TIMES: u8 = 10;

pub async fn gen_random_account_id(
    multi_sig_cli: &ContractClient<MultiSig>,
) -> Result<String, BackendError> {
    for _ in 0..FIND_VALID_ACCOUNT_RETRY_TIMES {
        let relayer_name = &common::env::CONF.relayer_pool.account_id;
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
    Err(BackendError::InternalError(
        "gen random account_id reach limit".to_string(),
    ))
}

pub async fn get_uncompleted_tx(account: &str) -> Result<Vec<CoinTxEntity>> {
    let mut txs = CoinTxEntity::find(CoinTxFilter::BySenderUncompleted(account)).await?;
    txs.retain(|tx| {
        tx.transaction.stage <= CoinSendStage::ReceiverApproved
            && now_millis() < tx.transaction.expire_at
    });
    Ok(txs)
}

pub async fn have_no_uncompleted_tx(account: &str) -> Result<(), BackendError> {
    let tx = get_uncompleted_tx(account).await?;
    if !tx.is_empty() {
        Err(WalletError::HaveUncompleteTx)?;
    }
    Ok(())
}

pub async fn get_freezn_amount(account: &str, coin: &MT) -> u128 {
    let mut tx = get_uncompleted_tx(account).await.unwrap();
    tx.retain(|x| x.transaction.coin_type == *coin);
    tx.iter().map(|x| x.transaction.amount).sum()
}

pub async fn get_available_amount(account_id: &str, coin: &MT) -> BackendRes<u128> {
    let coin_cli = ContractClient::<Coin>::new_query_cli(coin.clone())
        .await
        .map_err(|err| ChainError(err.to_string()))?;
    let balance = coin_cli
        .get_balance(account_id)
        .await
        .unwrap()
        .unwrap_or("0".to_string());
    let freezn_amount = get_freezn_amount(account_id, coin).await;
    let total: u128 = balance.parse().unwrap();
    if total < freezn_amount {
        Err(BackendError::InternalError(format!(
            "{}(total) more than {}(freezn_amount)",
            total, freezn_amount
        )))?
    } else {
        Ok(Some(total - freezn_amount))
    }
}

//calculate total value for dollar
//目前的场景转账超过300兆才会溢出
//由于取整造成的精度丢失可以忽略
pub async fn get_value(coin: &MT, amount: u128) -> u128 {
   //todo: get mt price
   1
}

pub async fn get_servant_need(strategy: &Vec<MultiSigRank>, coin: &MT, amount: u128) -> u8 {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let transfer_value = get_value(coin, amount).await;
    strategy
        .iter()
        .find(|&rank| transfer_value >= rank.min && transfer_value < rank.max_eq)
        .map(|rank| rank.sig_num)
        .unwrap_or(0)
}

//获取当前会话的用户信息、多签配置、设备信息的属性数据
//且已经进行过了多签
pub async fn get_session_state(
    user_id: u32,
    device_id: &str,
) -> Result<(UserInfo, StrategyData, DeviceInfo), BackendError> {
    let user = UserInfoEntity::find_single(UserFilter::ById(&user_id))
        .await
        .map_err(|err| {
            if err.to_string().contains("DBError::DataNotFound") {
                WalletError::MainAccountNotExist(err.to_string()).into()
            } else {
                BackendError::InternalError(err.to_string())
            }
        })?;

    let main_account = &user.user_info.main_account;
    let multi_sig_cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let current_strategy =
        multi_sig_cli
            .get_strategy(main_account)
            .await?
            .ok_or(BackendError::InternalError(
                "main_account not found".to_string(),
            ))?;

    //注册过的一定有设备信息
    let device = DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser(device_id, &user_id))
        .await?
        .into_inner();
    Ok((user.user_info, current_strategy, device))
}

pub fn check_role(current: KeyRole, require: KeyRole) -> Result<(), WalletError> {
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
