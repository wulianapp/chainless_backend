use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    account_manager::{UserFilter, UserInfoEntity},
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    utils::{get_user_context, token_auth},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInviteCodeRequest {
    code: String,
}

pub async fn req(req: HttpRequest, request_data: ChangeInviteCodeRequest) -> BackendRes<String> {
    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let role = context.role()?;
    check_role(role, KeyRole::Master)?;
    let ChangeInviteCodeRequest { code } = request_data;

    if code.len() < 4 || code.len() > 20 {
        Err(AirdropError::InviteCodeIllegal)?;
    }

    //todo: get kyc info
    let user_airdrop = AirdropEntity::find(AirdropFilter::ByInviteCode(&code), &mut db_cli).await?;
    if !user_airdrop.is_empty() {
        Err(AirdropError::InviteCodeAlreadyUsed)?;
    }

    AirdropEntity::update_single(
        AirdropUpdater::InviteCode(&code),
        AirdropFilter::ByUserId(&user_id),
        &mut db_cli,
    )
    .await?;

    Ok(None)
}
