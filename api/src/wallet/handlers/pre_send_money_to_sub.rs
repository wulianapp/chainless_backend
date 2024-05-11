use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, CoinTransaction, TxType};
use common::data_structures::CoinType;

use common::data_structures::KeyRole2;
use common::utils::math::coin_amount::display2raw;
use common::utils::time::{now_millis, DAY1};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::{debug, error};

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{parse_str, AccountManagerError, BackendError, BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::CoinTxView;
use models::PsqlOp;

use crate::wallet::PreSendMoneyToSubRequest;
use common::error_code::BackendError::ChainError;

//todo: DRY
pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreSendMoneyToSubRequest,
) -> BackendRes<(String, String)> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let PreSendMoneyToSubRequest {
        to,
        coin,
        amount,
        expire_at: _,
        memo,
    } = request_data;
    let expire_at = now_millis() + DAY1;
    let amount = display2raw(&amount).map_err(|err| BackendError::RequestParamInvalid(err))?;
    if amount == 0 {
        Err(WalletError::FobidTransferZero)?;
    }
    let coin_type = parse_str(&coin)?;
    let from = main_account.clone();

    let available_balance = super::get_available_amount(&from, &coin_type).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        error!(
            "{},  {}(amount)  big_than2 {}(available_balance) ",
            coin_type, amount, available_balance
        );
        Err(WalletError::InsufficientAvailableBalance)?;
    }
    error!(
        "{},  {}(amount)  big_than3 {}(available_balance) ",
        coin_type, amount, available_balance
    );

    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new().map_err(|err| ChainError(err.to_string()))?;
    let strategy = cli
        .get_strategy(&main_account)
        .await?
        .ok_or(BackendError::InternalError(
            "main_account not found".to_string(),
        ))?;
    if let Some(sub_conf) = strategy.sub_confs.get(&to) {
        debug!("to[{}] is subaccount of from[{}]", to, from);
        let coin_value = super::get_value(&coin_type, amount).await;
        let balance_value = cli.get_total_value(&to).await?;
        if coin_value + balance_value > sub_conf.hold_value_limit {
            Err(WalletError::ExceedSubAccountHoldLimit)?;
        }
    } else {
        Err(WalletError::ReceiverIsNotSubaccount)?;
    }

    let gen_tx_with_status =
        |stage: CoinSendStage| -> std::result::Result<CoinTxView, BackendError> {
            let coin_tx_raw = cli
                .gen_send_money_info(&from, &to, coin_type.clone(), amount, expire_at)
                .map_err(|err| ChainError(err.to_string()))?;
            Ok(CoinTxView::new_with_specified(
                coin_type.clone(),
                from.clone(),
                to.clone(),
                amount,
                coin_tx_raw,
                memo,
                expire_at,
                stage,
            ))
        };

    let need_sig_num = super::get_servant_need(&strategy.multi_sig_ranks, &coin_type, amount).await;

    //转子账户不需要is_forced标志位，本身就是强制的
    let mut coin_info = if need_sig_num == 0 {
        gen_tx_with_status(CoinSendStage::ReceiverApproved)?
    } else {
        gen_tx_with_status(CoinSendStage::Created)?
    };
    coin_info.transaction.tx_type = TxType::MainToSub;
    coin_info.insert()?;
    Ok(Some((
        coin_info.transaction.order_id,
        coin_info.transaction.coin_tx_raw,
    )))
}
