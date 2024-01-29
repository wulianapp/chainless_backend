use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{AccountManagerError, WalletError};
use common::http::{ApiRes, token_auth};
use models::coin_transfer::CoinTxFilter;
use crate::wallet::ReconfirmSendMoneyRequest;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> ApiRes<String, WalletError> {
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    //todo: check must be main device
    let ReconfirmSendMoneyRequest {
        device_id,
        tx_id,
        is_confirmed,
    } = request_data.0;

    let status = if is_confirmed {
        CoinTxStatus::SenderReconfirmed
    } else {
        CoinTxStatus::SenderCanceled
    };
    models::coin_transfer::update_status(status, CoinTxFilter::ByTxId(tx_id));
    Ok(None::<String>)
}