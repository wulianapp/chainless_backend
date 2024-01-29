use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{AccountManagerError, WalletError};
use common::http::{ApiRes, token_auth};
use models::account_manager;
use models::account_manager::UserFilter;
use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> ApiRes<String, WalletError> {
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    let mut coin_tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw).unwrap();
    coin_tx.status = CoinTxStatus::Created;
    if coin_tx.sender != user_id {
        Err(WalletError::TxFromNotBeUser)?;
    }

    //for receiver
    if let Some(user) = account_manager::get_user(UserFilter::ById(coin_tx.receiver)) {
        let _tx = models::coin_transfer::single_insert(&coin_tx).unwrap();
    } else {
        Err(WalletError::ReceiverNotFound)?;
    }
    Ok(None::<String>)
}