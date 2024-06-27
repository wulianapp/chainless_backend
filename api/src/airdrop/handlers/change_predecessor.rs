use actix_web::HttpRequest;

use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::data_structures::{airdrop::Airdrop, KeyRole};

use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{get_user_context, token_auth},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangePredecessorRequest {
    predecessor_invite_code: String,
}

pub async fn req(req: HttpRequest, request_data: ChangePredecessorRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;
    check_role(role, KeyRole::Master)?;

    let ChangePredecessorRequest {
        predecessor_invite_code,
    } = request_data;


    let predecessor_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByInviteCode(&predecessor_invite_code))
            .await
            .map_err(|_e| AirdropError::PredecessorInviteCodeNotExist)?;

    let Airdrop {
        user_id: predecessor_user_id,
        account_id: predecessor_account_id,
        ..
    } = predecessor_airdrop.airdrop;

    if predecessor_account_id.eq(&main_account) {
        Err(AirdropError::ForbidSetSelfAsPredecessor)?;
    }

    AirdropEntity::update_single(
        AirdropUpdater::Predecessor(
            &predecessor_user_id,
            &predecessor_account_id
        ),
        AirdropFilter::ByUserId(&user_id),
    )
    .await?;

    //predecessor must have called claim_dw20
    let mut cli = ContractClient::<ChainAirdrop>::new_update_cli().await?;
    let user_info = cli.get_user(&main_account).await?;
    if user_info.is_some() {
        cli.change_predecessor(&main_account, &predecessor_account_id)
            .await?;
    }
    Ok(None)
}
