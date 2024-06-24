use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::constants::TX_EXPAIRE_TIME;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};

use common::data_structures::KeyRole;
use common::utils::math::coin_amount::display2raw;
use common::utils::time::now_millis;

use tracing::{debug, error};

use crate::utils::{get_user_context, token_auth};
use common::error_code::{parse_str, BackendError, BackendRes, WalletError};

use common::error_code::BackendError::ChainError;
use models::coin_transfer::CoinTxEntity;
use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyToSubRequest {
    to: String,
    coin: String,
    amount: String,
    expire_at: u64,
    memo: Option<String>,
}

//todo: DRY
pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreSendMoneyToSubRequest,
) -> BackendRes<(String, String)> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;
    super::check_role(role, KeyRole::Master)?;

    let PreSendMoneyToSubRequest {
        to,
        coin,
        amount,
        expire_at: _,
        memo,
    } = request_data;
    let expire_at = now_millis() + TX_EXPAIRE_TIME;
    let amount = display2raw(&amount).map_err(|_e| WalletError::UnSupportedPrecision)?;
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
    let cli = ContractClient::<MultiSig>::new_query_cli()
        .await
        .map_err(|err| ChainError(err.to_string()))?;
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
        |stage: CoinSendStage| -> std::result::Result<CoinTxEntity, BackendError> {
            let coin_tx_raw = cli
                .gen_send_money_info(&from, &to, coin_type.clone(), amount, expire_at)
                .map_err(|err| ChainError(err.to_string()))?;
            Ok(CoinTxEntity::new_with_specified(
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
    let order_id = coin_info.transaction.order_id.clone();
    let coin_tx_raw = coin_info.transaction.order_id.clone();

    coin_info.transaction.tx_type = TxType::MainToSub;
    coin_info.insert().await?;
    Ok(Some((order_id, coin_tx_raw)))
}
