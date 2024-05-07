use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::bridge_on_near::Bridge;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{CoinType, KeyRole2};
use common::utils::math::coin_amount::display2raw;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::{debug, error};

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{AccountManagerError, BackendError, BackendRes, BridgeError, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::{CoinTxView};
use models::PsqlOp;

use crate::bridge::PreWithdrawRequest;
use crate::wallet::handlers::*;
use common::error_code::BackendError::ChainError;

//todo: DRY
pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreWithdrawRequest,
) -> BackendRes<(String, String)> {
    println!("__0001_start preWithdraw ");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    let (user, current_strategy, device) = get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let bridge_cli = ContractClient::<Bridge>::new()?;
    let eth_addr = bridge_cli.get_binded_eth_addr(&main_account).await?;
    let to = eth_addr.ok_or(BridgeError::NotBindEthAddr)?;

    let PreWithdrawRequest {
        coin,
        amount,
        expire_at,
        memo,
    } = request_data;

    let amount = display2raw(&amount).map_err(|err| BackendError::RequestParamInvalid(err))?;

    let coin_type = CoinType::from_str(&coin).map_err(|e| BackendError::RequestParamInvalid(e.to_string()))?;
    let from = main_account.clone();

    let available_balance = get_available_amount(&from, &coin_type).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        error!("{},  {}(amount)  big_than1 {}(available_balance) ",coin_type,amount,available_balance);
        Err(WalletError::InsufficientAvailableBalance)?;
    }

    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new().map_err(|err| ChainError(err.to_string()))?;
    let strategy = cli
        .get_strategy(&main_account)
        .await
        .map_err(|err| ChainError(err.to_string()))?
        .ok_or(WalletError::SenderNotFound)?;

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

    let need_sig_num = get_servant_need(&strategy.multi_sig_ranks, &coin_type, amount).await;

    //转跨链不需要is_forced标志位，本身就是强制的
    if need_sig_num == 0 {
        let mut coin_info = gen_tx_with_status(CoinSendStage::ReceiverApproved)?;
        let (tx_id, chain_tx_raw) = cli
            .gen_send_money_raw(vec![], &from, &to, coin_type, amount, expire_at)
            .await?;

        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.transaction.tx_id = Some(tx_id);
        coin_info.transaction.tx_type = TxType::MainToBridge;
        coin_info.insert()?;
        Ok(Some((
            coin_info.transaction.order_id,
            coin_info.transaction.coin_tx_raw,
        )))
    } else {
        let mut coin_info = gen_tx_with_status(CoinSendStage::Created)?;
        coin_info.transaction.tx_type = TxType::MainToBridge;
        coin_info.insert()?;
        Ok(None)
    }
}
