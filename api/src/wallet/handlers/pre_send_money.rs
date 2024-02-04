use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::{AddressConvert, CoinTransaction, CoinTxStatus, CoinType};
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::account_manager;
use models::account_manager::UserFilter;
use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: PreSendMoneyRequest,
) -> BackendRes<String> {
    //todo: allow master only
    let user_id =
        token_auth::validate_credentials(&req)?;
    let PreSendMoneyRequest{ device_id,
        from,
        to,
        coin,
        amount,
        expire_at,
        memo
    } = request_data;

    let coin_tx = CoinTransaction {
        tx_id: None,
        coin_type: CoinType::from_account_str(&coin).unwrap(),
        sender: from,
        receiver: to,
        amount,
        status: CoinTxStatus::Created,
        raw_data: None,
        signatures: vec![],
        memo,
        expire_at,
    };
    models::coin_transfer::single_insert(&coin_tx)?;
    Ok(None::<String>)
}