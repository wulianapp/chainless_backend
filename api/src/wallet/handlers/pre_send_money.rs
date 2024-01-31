use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::account_manager;
use models::account_manager::UserFilter;
use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> BackendRes<String> {
    let user_id =
        token_auth::validate_credentials(&req)?;

    let mut coin_tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw).unwrap();
    coin_tx.status = CoinTxStatus::Created;
    if coin_tx.sender != user_id {
        Err(TxFromNotBeUser)?;
    }

    //for receiver
    if let Some(user) = account_manager::get_user(UserFilter::ById(coin_tx.receiver))? {
        let _tx = models::coin_transfer::single_insert(&coin_tx)?;
    } else {
        Err(ReceiverNotFound)?;
    }
    Ok(None::<String>)
}