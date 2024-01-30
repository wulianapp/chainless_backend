use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{ApiCommonError::*, WalletError::*};
use common::http::{ApiRes, token_auth};
use crate::wallet::DirectSendMoneyRequest;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: web::Json<DirectSendMoneyRequest>,
) -> ApiRes<String>{
    //todo: must be called by main device
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| Authorization(e).into())?;

    let mut coin_tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw).unwrap();
    coin_tx.status = CoinTxStatus::Created;
    if coin_tx.sender != user_id {
        Err(TxFromNotBeUser.into())?;
    }
    //todo: update_status && collect other sign
    Ok(None::<String>)
}