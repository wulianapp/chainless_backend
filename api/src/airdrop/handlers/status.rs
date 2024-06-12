use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{airdrop::Airdrop, wallet_namage_record::WalletOperateType, KeyRole},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::utils::{get_user_context, token_auth, wallet_grades::query_wallet_grade};
use crate::wallet::handlers::*;
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
    //pub cly_claimed: Option<String>,
    //pub du20_claimed: Option<String>,
}

pub async fn req(req: HttpRequest) -> BackendRes<AirdropStatusResponse> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let _ = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    //todo: check sig,
    
    //todo: get kyc info
    let user_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id)).await?;
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
    Ok(Some(AirdropStatusResponse {
        user_id: user_id.to_string(),
        account_id,
        invite_code,
        predecessor_user_id: predecessor_user_id.to_string(),
        predecessor_account_id,
        btc_address,
        btc_level,
        //cly_claimed: None,
        //du20_claimed: None,
    }))
}
