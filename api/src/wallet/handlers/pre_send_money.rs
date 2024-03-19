use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::debug;

use crate::utils::token_auth;
use common::error_code::{AccountManagerError, BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::{get_next_tx_index, CoinTxView};
use models::PsqlOp;

use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(req: HttpRequest, request_data: PreSendMoneyRequest) -> BackendRes<(u32,String)> {
    //todo: allow master only
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let PreSendMoneyRequest {
        from,
        to,
        coin,
        amount,
        expire_at,
        memo,
        is_forced
    } = request_data;
    let coin_type = CoinType::from_str(&coin).unwrap();

    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }
    let available_balance = super::get_available_amount(&from,&coin_type).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        Err(WalletError::InsufficientAvailableBalance)?;
    }
    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new();
    let strategy = cli
        .get_strategy(&user_info.user_info.main_account)
        .await?
        .ok_or(WalletError::SenderNotFound)?;
    if let Some(sub_conf) = strategy.sub_confs.get(&to){
        debug!("to[{}] is subaccount of from[{}]",to,from);
        let coin_price = 1;
        let balance_value = cli.get_total_value(&to).await;
        if  amount * coin_price + balance_value > sub_conf.hold_value_limit {
            Err(WalletError::ExceedSubAccountHoldLimit)?;
        }
    }

    let tx_status = if strategy.servant_pubkeys.is_empty() {
        if let Some(_) = strategy.sub_confs.get(&to){
            CoinTxStatus::SenderSigCompletedAndReceiverIsSub
        }else if is_forced{
            CoinTxStatus::ReceiverApproved
        }else{                
            CoinTxStatus::SenderSigCompleted
        }
    } else {
        CoinTxStatus::Created
    };

    


    let coin_tx_raw = cli
        .gen_send_money_info(&from, &to, coin_type.clone(), amount, expire_at)
        .unwrap();
    let mut coin_info = CoinTxView::new_with_specified(
        coin_type.clone(),
        from.clone(),
        to.clone(),
        amount,
        coin_tx_raw,
        memo,
        expire_at,
        tx_status,
    );
    if is_forced {
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
        .unwrap()
        .unwrap();
        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.insert()?;
        Ok(Some((next_tx_index,tx_id)))
    }else {
        coin_info.insert()?;
        Ok(None)
    }

}
