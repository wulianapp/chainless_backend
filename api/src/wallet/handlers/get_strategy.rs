use actix_web::HttpRequest;
use serde::Serialize;
use blockchain::multi_sig::{MultiSig, StrategyData};
use common::error_code::{AccountManagerError, BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::account_manager::{get_user, UserFilter};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use crate::wallet::searchMessageByAccountIdRequest;


pub(crate) async fn req(req: HttpRequest,request_data: searchMessageByAccountIdRequest) -> BackendRes<StrategyData> {
    let user_id =
        token_auth::validate_credentials(&req)?;

    let strategy = blockchain::ContractClient::<MultiSig>::new();
    let strategy = strategy.get_strategy(&request_data.account_id).await.unwrap();

    Ok(Some(strategy))
}