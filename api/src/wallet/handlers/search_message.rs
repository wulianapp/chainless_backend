use actix_web::HttpRequest;
use serde::Serialize;
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::coin_transfer::{CoinTxFilter, CoinTxView};

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<CoinTxView>> {
    let user_id =
        token_auth::validate_credentials(&req)?;

    println!("searchMessage user_id {}", user_id);
    let message = models::coin_transfer::get_transactions(CoinTxFilter::ByUserPending(user_id))?;
    Ok(Some(message))
}