use actix_web::{web, HttpRequest};

use blockchain::{
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    account_manager::{UserFilter, UserInfoView}, airdrop::{AirdropFilter, AirdropUpdater, AirdropView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PsqlOp
};
use serde::{Deserialize,Serialize};
use tracing::{debug, info};

use crate::wallet::handlers::*;
use crate::wallet::UpdateStrategy;
use crate::{
    utils::{token_auth, wallet_grades::query_wallet_grade},
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInviteCodeRequest {
    code: String
}

pub async fn req(req: HttpRequest, request_data: ChangeInviteCodeRequest) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let user = UserInfoView::find_single(UserFilter::ById(user_id), &mut pg_cli).await?;
    if user.user_info.main_account.ne("") {
        let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
        let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
        check_role(current_role, KeyRole2::Master)?;
    }
    let ChangeInviteCodeRequest { code} = request_data;

    if code.len() < 4 || code.len() > 20 {
        Err(AirdropError::InviteCodeIllegal)?;
    }

    //todo: get kyc info
    let user_airdrop = AirdropView::find(
        AirdropFilter::ByInviteCode(&code), 
        &mut pg_cli
    ).await?;
    if user_airdrop.len() != 0{
        Err(AirdropError::InviteCodeAlreadyUsed)?;
    }

    AirdropView::update_single(
        AirdropUpdater::InviteCode(&code),
         AirdropFilter::ByUserId(&user_id.to_string()),
         &mut pg_cli
    ).await?;

    Ok(None)
}
