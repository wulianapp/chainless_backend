use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, StrategyData};

use common::http::{token_auth, BackendRes};

use crate::wallet::{getStrategyRequest, searchMessageByAccountIdRequest};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: getStrategyRequest,
) -> BackendRes<StrategyData> {
    let _user_id = token_auth::validate_credentials(&req)?;

    let strategy = blockchain::ContractClient::<MultiSig>::new();
    let strategy = strategy
        .get_strategy(&request_data.account_id)
        .await
        .unwrap();

    Ok(Some(strategy))
}
