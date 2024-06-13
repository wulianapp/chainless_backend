use actix_web::HttpRequest;
use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::data_structures::KeyRole;
use models::{
    airdrop::{AirdropEntity, AirdropFilter},
    PsqlOp,
};
use tracing::debug;

use crate::utils::{get_user_context, token_auth};
use crate::wallet::handlers::*;
use blockchain::ContractClient;
use common::error_code::BackendRes;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device
    //todo: sync tx records after claim

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    let user_airdrop = AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id)).await?;

    //上级必须也领过空投
    let cli = ContractClient::<ChainAirdrop>::new_query_cli().await?;
    let predecessor_airdrop_on_chain = cli
        .get_user(&user_airdrop.airdrop.predecessor_account_id)
        .await?;
    if predecessor_airdrop_on_chain.is_none() {
        Err(AirdropError::PredecessorHaveNotClaimAirdrop)?;
    }

    let cli = ContractClient::<ChainAirdrop>::new_update_cli().await?;
    let ref_user = cli
        .claim_dw20(
            &main_account,
            &user_airdrop.airdrop.predecessor_account_id,
            user_airdrop.airdrop.btc_address.as_deref(),
            user_airdrop.airdrop.btc_level.unwrap_or_default(),
        )
        .await?;
    debug!("successful claim dw20 txid {}", ref_user);
    Ok(None)
}
