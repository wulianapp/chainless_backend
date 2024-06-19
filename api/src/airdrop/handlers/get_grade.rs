use actix_web::HttpRequest;

use serde::{Deserialize, Serialize};

use crate::{
    utils::{token_auth, wallet_grades::query_wallet_grade}
};
use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetGradeRequest {
    btc_address: String,
}

pub async fn req(req: HttpRequest, request_data: GetGradeRequest) -> BackendRes<u8> {
    let _ = token_auth::validate_credentials(&req).await?;

    let GetGradeRequest {
        btc_address,
    } = request_data;
    let grade = query_wallet_grade(&btc_address).await?;
    Ok(Some(grade))
}
