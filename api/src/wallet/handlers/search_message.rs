use actix_web::HttpRequest;
use serde::Serialize;
use common::error_code::{AccountManagerError, WalletError};
use common::http::{ApiRes, token_auth};
use models::coin_transfer::{CoinTxFilter, CoinTxView};

pub(crate) async fn req(req: HttpRequest) -> ApiRes<Vec<CoinTxView>, WalletError> {
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    println!("searchMessage user_id {}", user_id);
    let message = models::coin_transfer::get_transactions(CoinTxFilter::ByUserPending(user_id));
    Ok(Some(message))
}