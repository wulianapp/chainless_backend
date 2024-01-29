use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{AccountManagerError, WalletError};
use common::http::{ApiRes, token_auth};
use models::coin_transfer::CoinTxFilter;
use crate::wallet::ReactPreSendMoney;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: web::Json<ReactPreSendMoney>,
) -> ApiRes<String, WalletError> {
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    let ReactPreSendMoney { tx_id, is_agreed } = request_data.0;
    //message max is 10，
    //let FinalizeSha = request_data.clone();
    let status = if is_agreed {
        CoinTxStatus::ReceiverApproved
    } else {
        CoinTxStatus::ReceiverRejected
    };
    models::coin_transfer::update_status(status, CoinTxFilter::ByTxId(tx_id));
    Ok(None::<String>)
}