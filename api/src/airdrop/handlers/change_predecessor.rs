use actix_web::{web, HttpRequest};

use blockchain::{
    airdrop::Airdrop as ChainAirdrop,
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use lettre::transport::smtp::client;
use models::{
    airdrop::{AirdropFilter, AirdropUpdater, AirdropView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PsqlOp
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
    predecessor_account_id: String,
}

pub async fn req(req: HttpRequest, request_data: ChangePredecessorRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut pg_cli).await?;

    if main_account == "".to_string(){
       Err(BackendError::InternalError("".to_string()))?; 
    }

    let ChangePredecessorRequest { predecessor_account_id} = request_data;
    //fixme：是否允许安全问答之前进行修改
    AirdropView::update_single(
        AirdropUpdater::predecessor(&predecessor_account_id),
         AirdropFilter::ByUserId(&user_id.to_string()), 
         &mut pg_cli
    ).await?;

    let cli = ContractClient::<ChainAirdrop>::new().await?;
    cli.change_predecessor(&main_account,&predecessor_account_id).await?;


    //todo: change ref
    Ok(None)
}
