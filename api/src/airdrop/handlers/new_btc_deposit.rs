use actix_web::{web, HttpRequest};
use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
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
use tracing::{debug, info, warn};

use crate::utils::{token_auth, wallet_grades::query_wallet_grade};
use crate::wallet::handlers::*;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewBtcDepositRequest {
    sender: String,
    receiver: String,
}

pub async fn req(_req: HttpRequest, request_data: NewBtcDepositRequest) -> BackendRes<String> {
    let mut db_cli = get_pg_pool_connect().await?;

    //todo: 目前该接口不做限制，后续看怎么收拢权限
    let NewBtcDepositRequest { sender, receiver } = request_data;

    let airdrop_info =
        AirdropEntity::find(AirdropFilter::ByBtcAddress(&receiver), &mut db_cli).await?;
    if airdrop_info.is_empty() {
        //Err(BackendError::InternalError("".to_string()))?;
        warn!("receiver {} isn't belong us", receiver);
        return Ok(None);
    }

    //不允许重复评级
    if airdrop_info.len() == 1
        && airdrop_info[0].airdrop.btc_address.is_some()
        && airdrop_info[0].airdrop.btc_address.is_none()
    {
        let grade = query_wallet_grade(&sender).await?;
        AirdropEntity::update_single(
            AirdropUpdater::BtcLevel(grade),
            AirdropFilter::ByBtcAddress(&receiver),
            &mut db_cli,
        )
        .await?;
        info!(
            "check deposit(sender={},receiver={}) sucessfully,and get grade  {}",
            sender, receiver, grade
        );
    } else {
        warn!("deposit from {} is invaild", sender);
    }

    Ok(None)
}
