use actix_web::{web, HttpRequest};

use crate::wallet::DirectSendMoneyRequest;
use common::http::{token_auth, BackendRes};

pub(crate) async fn req(
    req: HttpRequest,
    _request_data: web::Json<DirectSendMoneyRequest>,
) -> BackendRes<String> {
    //todo: must be called by main device
    let _user_id = token_auth::validate_credentials(&req)?;

    /***
    let mut coin_tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw).unwrap();
    coin_tx.status = CoinTxStatus::Created;
    if coin_tx.sender != user_id {
        Err(TxFromNotBeUser)?;
    }

     */
    //todo: update_status && collect other sign
    Ok(None::<String>)
}
