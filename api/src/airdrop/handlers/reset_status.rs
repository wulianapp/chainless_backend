use actix_web::HttpRequest;
use airdrop::BtcGradeStatus;
use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::data_structures::KeyRole;
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::utils::{get_user_context, token_auth};
use crate::wallet::handlers::*;
use blockchain::ContractClient;
use common::error_code::BackendRes;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: sync tx records after claim

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    //todo: 链上必须没有
    let cli = ContractClient::<ChainAirdrop>::new_query_cli().await?;
    let predecessor_airdrop_on_chain = cli.get_user(&main_account).await?;
    if predecessor_airdrop_on_chain.is_some() {
        Err(AirdropError::AlreadyClaimedDw20)?;
    }

    //一旦选择了不使用btc地址，则把之前的清掉,状态为Reconfirmed
    AirdropEntity::update_single(
        AirdropUpdater::ResetBind,
        AirdropFilter::ByAccountId(&main_account),
    )
    .await?;

    Ok(None)
}
