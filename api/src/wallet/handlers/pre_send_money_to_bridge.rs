use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType, TxType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{AccountManagerError, BackendError, BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::{get_next_tx_index, CoinTxView};
use models::PsqlOp;

use crate::wallet::PreSendMoneyToBridgeRequest;
use common::error_code::BackendError::ChainError;


//todo: DRY
pub(crate) async fn req(req: HttpRequest, request_data: PreSendMoneyToBridgeRequest) -> BackendRes<(u32,String)> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    let (user,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let main_account = user.main_account;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Master)?;


    let PreSendMoneyToBridgeRequest {
        coin,
        amount,
        expire_at,
        memo,
        captcha
    } = request_data;
    let coin_type = CoinType::from_str(&coin).unwrap();
    let from = main_account.clone();
    let to = common::env::CONF.bridge_near_contract.clone();

    let available_balance = super::get_available_amount(&from,&coin_type).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        Err(WalletError::InsufficientAvailableBalance)?;
    }

    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new().map_err(|err| ChainError(err.to_string()))?;
    let strategy = cli
        .get_strategy(&main_account)
        .await.map_err(|err| ChainError(err.to_string()))?
        .ok_or(WalletError::SenderNotFound)?;

    let gen_tx_with_status = |status: CoinTxStatus| -> std::result::Result<CoinTxView,BackendError>{
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
            status,
        ))
    };

    let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &coin_type,
        amount
    ).await; 

    //转子账户不需要is_forced标志位，本身就是强制的
    if need_sig_num == 0 {
        if captcha.is_none(){
            Err(BackendError::InternalError("For single tx,need captcha".to_string()))?;
        } 
        Captcha::check_user_code(&user_id.to_string(), &captcha.unwrap(), Usage::PreSendMoneyToBridge)?;

        let mut coin_info = gen_tx_with_status( CoinTxStatus::SenderSigCompletedAndReceiverIsBridge)?;
        let next_tx_index = get_next_tx_index()?;
        let (tx_id, chain_tx_raw) = cli
        .gen_send_money_raw(
            next_tx_index as u64,
            vec![],
            &from,
            &to,
            coin_type,
            amount,
            expire_at,
        )
        .await
        .map_err(|err| ChainError(err.to_string()))?;
        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.transaction.tx_type = TxType::MainToSub;
        coin_info.insert()?;
        Ok(Some((next_tx_index,tx_id)))
    }else {
        //todo:
        if captcha.is_some(){
            Err(BackendError::InternalError("For multi-sig tx,need not  captcha".to_string()))?;
        }
        let mut coin_info = gen_tx_with_status(CoinTxStatus::Created)?;
        coin_info.transaction.tx_type = TxType::MainToBridge;
        coin_info.insert()?;
        Ok(None)
    }
}
