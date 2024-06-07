use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::multi_sig::{CoinTx, MultiSig};
use blockchain::ContractClient;
use common::constants::TX_EXPAIRE_TIME;
use common::data_structures::coin_transaction::{CoinSendStage, CoinTransaction, TxType};
use common::data_structures::CoinType;

use common::data_structures::KeyRole2;
use common::utils::math::coin_amount::display2raw;
use common::utils::time::{now_millis, DAY1};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::{get_user_context, token_auth};
use common::error_code::{
    to_param_invalid_error, AccountManagerError, BackendError, BackendRes,
    WalletError::{self, *},
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoEntity};

use models::coin_transfer::CoinTxEntity;
use models::PsqlOp;

use anyhow::Result;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    to: String,
    coin: String,
    amount: String,
    expire_at: u64,
    memo: Option<String>,
    is_forced: bool,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreSendMoneyRequest,
) -> BackendRes<(String, Option<String>)> {
    //todo: allow master only
    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, _,device_id,_) = token_auth::validate_credentials(&req,&mut db_cli).await?;
    let PreSendMoneyRequest {
        to,
        coin,
        amount,
        expire_at: _,
        memo,
        is_forced,
    } = request_data;
    let expire_at = now_millis() + TX_EXPAIRE_TIME;
    let amount = display2raw(&amount).map_err(|_e| WalletError::UnSupportedPrecision)?;
    if amount == 0 {
        Err(WalletError::FobidTransferZero)?;
    }
  
    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (main_account, strategy) = context.account_strategy()?;
    let role = context.role()?;
    
    super::check_role(role, KeyRole2::Master)?;

    //todo:
    let (to_account_id, to_contact) = if to.contains('@') || to.contains('+') {
        let receiver = UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&to), &mut db_cli)
            .await
            .map_err(|err| {
                if err.to_string().contains("DBError::DataNotFound") {
                    AccountManagerError::PhoneOrEmailNotRegister.into()
                } else {
                    BackendError::InternalError(err.to_string())
                }
            })?
            .into_inner();

        if receiver.main_account.is_none() {
            Err(WalletError::ReceiverNotSetSecurity)?;
        }
        (receiver.main_account.unwrap(), Some(to))
    } else {
        let _receiver = UserInfoEntity::find_single(UserFilter::ByMainAccount(&to), &mut db_cli)
            .await
            .map_err(|err| {
                if err.to_string().contains("DBError::DataNotFound") {
                    WalletError::MainAccountNotExist(err.to_string()).into()
                } else {
                    BackendError::InternalError(err.to_string())
                }
            })?;
        (to, None)
    };
    if to_account_id == main_account {
        Err(WalletError::ForbideTransferSelf)?
    }
    let coin_type = coin.parse().map_err(to_param_invalid_error)?;

    let available_balance = super::get_available_amount(&main_account, &coin_type, &mut db_cli).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        error!(
            "{},  {}(amount)  big_than1 {}(available_balance) ",
            coin_type, amount, available_balance
        );
        Err(WalletError::InsufficientAvailableBalance)?;
    }

    //如果本身是单签，则状态直接变成SenderSigCompleted
    if strategy.sub_confs.get(&to_account_id).is_some() {
        Err(WalletError::ReceiverIsSubaccount)?;
    }
    //todo: 也不能转bridge

    //封装根据状态生成转账对象的逻辑
    let cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let gen_tx_with_status = |stage: CoinSendStage| -> Result<CoinTxEntity> {
        let coin_tx_raw =
            cli.gen_send_money_info(&main_account, &to_account_id, coin_type.clone(), amount, expire_at)?;
        Ok(CoinTxEntity::new_with_specified(
            coin_type.clone(),
            main_account.clone(),
            to_account_id.clone(),
            amount,
            coin_tx_raw,
            memo,
            expire_at,
            stage,
        ))
    };

    let need_sig_num = super::get_servant_need(&strategy.multi_sig_ranks, &coin_type, amount).await;

    //fixme: this is unsafe
    debug!(
        "before create order {},{},{}",
        line!(),
        need_sig_num,
        is_forced
    );
    //没有从公钥且强制转账的话，直接返回待签名数据
    info!("need_sig_num: {},is_forced {} ", need_sig_num, is_forced);
    //单签 + 强制
    if need_sig_num == 0 && is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::ReceiverApproved)?;

        let (tx_id, chain_tx_raw) = cli
            .gen_send_money_raw(vec![], &main_account, &to_account_id, coin_type, amount, expire_at)
            .await?;
        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.transaction.tx_id = Some(tx_id.clone());
        coin_info.transaction.tx_type = TxType::Forced;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        let order_id = coin_info.transaction.order_id.clone();
        coin_info.insert(&mut db_cli).await?;
        Ok(Some((order_id, Some(tx_id))))
    //单签 + 非强制    
    } else if need_sig_num == 0 && !is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::SenderSigCompleted)?;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        let order_id = coin_info.transaction.order_id.clone();
        coin_info.insert(&mut db_cli).await?;
        Ok(Some((order_id, None)))
    //多签 + 强制    
    } else if need_sig_num != 0 && is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::Created)?;
        coin_info.transaction.tx_type = TxType::Forced;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        let order_id = coin_info.transaction.order_id.clone();
        coin_info.insert(&mut db_cli).await?;
        Ok(Some((order_id, None)))
    //多签 + 非强制    
    } else if need_sig_num != 0 && !is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::Created)?;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        let order_id = coin_info.transaction.order_id.clone();
        coin_info.insert(&mut db_cli).await?;
        Ok(Some((order_id, None)))
    } else {
        unreachable!("all case is considered")
    }
}
