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

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;
    check_role(role, KeyRole::Master)?;

 
    let airdrop_data =  AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id)).await?.into_inner();
    if airdrop_data.btc_grade_status != BtcGradeStatus::Calculated{
        Err(AirdropError::BtcGradeStatusIllegal)?;
    } 

    AirdropEntity::update_single(
        AirdropUpdater::GradeStatus(BtcGradeStatus::Reconfirmed),
        AirdropFilter::ByAccountId(&main_account),
    )
    .await?;

    Ok(None)
}
