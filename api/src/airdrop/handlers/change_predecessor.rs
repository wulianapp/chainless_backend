use actix_web::{web, HttpRequest};

use blockchain::{
    airdrop::Airdrop as ChainAirdrop,
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{airdrop::Airdrop, wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use lettre::transport::smtp::client;
use models::{
    airdrop::{AirdropFilter, AirdropUpdater, AirdropView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PgLocalCli, PsqlOp
};
use serde::{Deserialize, Serialize};
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
pub struct ChangePredecessorRequest {
    predecessor_invite_code: String,
}

pub async fn req(req: HttpRequest, request_data: ChangePredecessorRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut pg_cli = pg_cli.begin().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut pg_cli).await?;

    let ChangePredecessorRequest { predecessor_invite_code} = request_data;

    //todo: check predecessor_account_id if exist
    //todo： check if called claim_dw20
    
    let predecessor_airdrop = AirdropView::find_single(
        AirdropFilter::ByInviteCode(&predecessor_invite_code), 
        &mut pg_cli
    )
    .await
    .map_err(|_e| AirdropError::PredecessorInviteCodeNotExist)?;

    let Airdrop{
        user_id: predecessor_user_id,
        account_id:predecessor_account_id ,..
    } = predecessor_airdrop.airdrop;

    if predecessor_account_id.as_ref().unwrap().eq(&main_account){
        Err(AirdropError::ForbidSetSelfAsPredecessor)?;
    }

    AirdropView::update_single(
        AirdropUpdater::Predecessor(&predecessor_user_id,predecessor_account_id.as_ref().unwrap()),
         AirdropFilter::ByUserId(&user_id.to_string()), 
         &mut pg_cli
    ).await?;

    let cli = ContractClient::<ChainAirdrop>::new().await?;
    cli.change_predecessor(&main_account,predecessor_account_id.as_ref().unwrap()).await?;

    pg_cli.commit().await?;
    //todo: change ref
    Ok(None)
}
