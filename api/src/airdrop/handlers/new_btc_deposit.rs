use actix_web::{web, HttpRequest};
use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropFilter, AirdropUpdater, AirdropView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PsqlOp
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::wallet::handlers::*;
use crate::{
    utils::{token_auth, wallet_grades::query_wallet_grade},
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewBtcDepositRequest {
    sender: String,
    receiver: String,
}

pub async fn req(req: HttpRequest,request_data: NewBtcDepositRequest) -> BackendRes<String> {
    let mut db_cli = get_pg_pool_connect().await?;

    //todo: 目前该接口不做限制，后续看怎么收拢权限
    let NewBtcDepositRequest{sender,receiver} = request_data;

    let airdrop_info = AirdropView::find(
        AirdropFilter::ByBtcAddress(&receiver),
        &mut db_cli
    ).await?;
    if airdrop_info.is_empty(){
        //Err(BackendError::InternalError("".to_string()))?;
        warn!("receiver {} isn't belong us",receiver);
        return Ok(None);
    }


    let grade = query_wallet_grade(&sender).await?;
    AirdropView::update_single(
        AirdropUpdater::BtcLevel(grade), 
        AirdropFilter::ByBtcAddress(&receiver), 
        &mut db_cli
    ).await?;
    info!("check deposit(sender={},receiver={}) sucessfully,and get grade  {}",
    sender,receiver,grade);
    Ok(None)
}
