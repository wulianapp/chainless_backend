use actix_web::{web, HttpRequest};

use blockchain::{
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{airdrop::Airdrop, wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropFilter, AirdropUpdater, AirdropView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PsqlOp
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AirdropStatusResponse {
    pub user_id: String,
    pub account_id: Option<String>,   
    pub invite_code: String, 
    pub predecessor_user_id: String, 
    pub predecessor_account_id: String, 
    pub btc_address: Option<String>,      
    pub btc_level: Option<u8>,
    pub cly_claimed: Option<String>,       
    pub du20_claimed: Option<String>,
}

pub async fn req(req: HttpRequest) -> BackendRes<AirdropStatusResponse> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut pg_cli).await?;


    //todo: check sig,
    //todo: get kyc info
    let user_airdrop = AirdropView::find_single(
        AirdropFilter::ByUserId(&user_id.to_string()), 
        &mut pg_cli
    ).await?;
    let Airdrop { 
        user_id, 
        account_id, 
        invite_code,
        predecessor_user_id,
        predecessor_account_id,
        btc_address, 
        btc_level, 
        ..
     } = user_airdrop.airdrop.clone();
    Ok(Some(AirdropStatusResponse{
        user_id,
        account_id,
        invite_code,
        predecessor_user_id,
        predecessor_account_id,
        btc_address,
        btc_level,
        cly_claimed: None,
        du20_claimed: None,
    }))
}