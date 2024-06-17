use actix_web::HttpRequest;

use airdrop::BtcGradeStatus;
use blockchain::airdrop::Airdrop;
use common::data_structures::KeyRole;
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{get_user_context, token_auth, wallet_grades::query_wallet_grade},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::BackendRes;
use strum_macros::{Display, EnumString};

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
