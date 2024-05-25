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
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::get_pg_pool_connect;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{
    to_param_invalid_error, AccountManagerError, BackendError, BackendRes,
    WalletError::{self, *},
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::CoinTxView;
use models::PsqlOp;

use crate::wallet::PreSendMoneyRequest;
use anyhow::Result;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreSendMoneyRequest,
) -> BackendRes<(String, Option<String>)> {
    //todo: allow master only
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let PreSendMoneyRequest {
        to,
        coin,
        amount,
        expire_at: _,
        memo,
        is_forced,
    } = request_data;
    let expire_at = now_millis() + TX_EXPAIRE_TIME;
    let amount = display2raw(&amount).map_err(BackendError::RequestParamInvalid)?;
    if amount == 0 {
        Err(WalletError::FobidTransferZero)?;
    }
    let mut pg_cli = get_pg_pool_connect().await?;
    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut pg_cli).await?;

    let (to_account_id, to_contact) = if to.contains('@') || to.contains('+') {
        let receiver = UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&to), &mut pg_cli)
            .await
            .map_err(|err| {
                if err.to_string().contains("DBError::DataNotFound") {
                    AccountManagerError::PhoneOrEmailNotRegister.into()
                } else {
                    BackendError::InternalError(err.to_string())
                }
            })?;

        if !receiver.user_info.secruity_is_seted {
            Err(WalletError::ReceiverNotSetSecurity)?;
        }
        (receiver.user_info.main_account, Some(to))
    } else {
        let _receiver = UserInfoView::find_single(UserFilter::ByMainAccount(&to), &mut pg_cli)
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
    if to_account_id == user.main_account {
        Err(WalletError::ForbideTransferSelf)?
    }

    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let from = user.main_account.clone();
    let coin_type = coin.parse().map_err(to_param_invalid_error)?;

    let available_balance = super::get_available_amount(&from, &coin_type, &mut pg_cli).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        error!(
            "{},  {}(amount)  big_than1 {}(available_balance) ",
            coin_type, amount, available_balance
        );
        Err(WalletError::InsufficientAvailableBalance)?;
    }

    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new().await?;
    let strategy = cli
        .get_strategy(&from)
        .await?
        .ok_or(BackendError::InternalError(
            "main_account not found".to_string(),
        ))?;
    if strategy.sub_confs.get(&to_account_id).is_some() {
        Err(WalletError::ReceiverIsSubaccount)?;
    }
    //todo: 也不能转bridge

    //封装根据状态生成转账对象的逻辑
    let gen_tx_with_status = |stage: CoinSendStage| -> Result<CoinTxView> {
        let coin_tx_raw =
            cli.gen_send_money_info(&from, &to_account_id, coin_type.clone(), amount, expire_at)?;
        Ok(CoinTxView::new_with_specified(
            coin_type.clone(),
            from.clone(),
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
    if need_sig_num == 0 && is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::ReceiverApproved)?;

        let (tx_id, chain_tx_raw) = cli
            .gen_send_money_raw(vec![], &from, &to_account_id, coin_type, amount, expire_at)
            .await?;
        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.transaction.tx_id = Some(tx_id.clone());
        coin_info.transaction.tx_type = TxType::Forced;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }

        coin_info.insert(&mut pg_cli).await?;
        Ok(Some((coin_info.transaction.order_id, Some(tx_id))))
    } else if need_sig_num == 0 && !is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::SenderSigCompleted)?;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        coin_info.insert(&mut pg_cli).await?;
        Ok(Some((coin_info.transaction.order_id, None)))
    } else if need_sig_num != 0 && is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::Created)?;
        coin_info.transaction.tx_type = TxType::Forced;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        coin_info.insert(&mut pg_cli).await?;
        Ok(Some((coin_info.transaction.order_id, None)))
    } else if need_sig_num != 0 && !is_forced {
        let mut coin_info = gen_tx_with_status(CoinSendStage::Created)?;
        if to_contact.is_some() {
            coin_info.transaction.receiver_contact = to_contact;
        }
        coin_info.insert(&mut pg_cli).await?;
        Ok(Some((coin_info.transaction.order_id, None)))
    } else {
        unreachable!("all case is considered")
    }
}
