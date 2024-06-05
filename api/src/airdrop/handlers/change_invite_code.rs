use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
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

use crate::{utils::token_auth, wallet::handlers::*};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInviteCodeRequest {
    code: String,
}

pub async fn req(req: HttpRequest, request_data: ChangeInviteCodeRequest) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let user = UserInfoEntity::find_single(UserFilter::ById(&user_id), &mut db_cli).await?;
    if user.user_info.main_account.is_some() {
        let (_user, current_strategy, device) =
            get_session_state(user_id, &device_id, &mut db_cli).await?;
        let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
        check_role(current_role, KeyRole2::Master)?;
    }
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
