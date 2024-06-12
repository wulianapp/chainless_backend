use actix_web::HttpRequest;



use blockchain::{
    fees_call::FeesCall,
};

use crate::utils::token_auth;

use common::{data_structures::CoinType, error_code::BackendRes};


pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<CoinType>> {
    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    let main_account = super::get_main_account(user_id).await?;
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new_query_cli().await?;

    let fees_priority = fees_call_cli.get_fees_priority(&main_account).await?;
    Ok(Some(fees_priority))
}
