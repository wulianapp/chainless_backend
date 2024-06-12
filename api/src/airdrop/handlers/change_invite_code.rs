use actix_web::{HttpRequest};


use blockchain::{airdrop::Airdrop, ContractClient};
use common::{
    data_structures::{KeyRole},
};
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};


use crate::{
    utils::{get_user_context, token_auth},
    wallet::handlers::*,
};

use common::error_code::{BackendRes};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInviteCodeRequest {
    code: String,
}

pub async fn req(req: HttpRequest, request_data: ChangeInviteCodeRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let role = context.role()?;
    check_role(role, KeyRole::Master)?;

    let cli = ContractClient::<Airdrop>::new_query_cli().await?;
    let user_airdrop_on_chain = cli.get_user(
        context.user_info.main_account.as_ref().unwrap()
    ).await?;
    
    if user_airdrop_on_chain.is_none(){
        Err(AirdropError::HaveNotClaimAirdrop)?;
    }


    let ChangeInviteCodeRequest { code } = request_data;

    if code.len() < 4 || code.len() > 20 {
        Err(AirdropError::InviteCodeIllegal)?;
    }

    //todo: get kyc info
    let user_airdrop = AirdropEntity::find(AirdropFilter::ByInviteCode(&code)).await?;
    if !user_airdrop.is_empty() {
        Err(AirdropError::InviteCodeAlreadyUsed)?;
    }

    AirdropEntity::update_single(
        AirdropUpdater::InviteCode(&code),
        AirdropFilter::ByUserId(&user_id),
    )
    .await?;

    Ok(None)
}
