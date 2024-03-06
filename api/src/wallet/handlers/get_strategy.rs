use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, StrategyData};

use common::error_code::{BackendRes};
use crate::utils::token_auth;


use crate::wallet::{getStrategyRequest};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: getStrategyRequest,
) -> BackendRes<StrategyData> {
    let _user_id = token_auth::validate_credentials(&req)?;

    let strategy = blockchain::ContractClient::<MultiSig>::new();
    strategy
        .get_strategy(&request_data.account_id)
        .await
}
