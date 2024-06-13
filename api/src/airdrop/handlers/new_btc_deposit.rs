use actix_web::HttpRequest;

use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::utils::wallet_grades::query_wallet_grade;

use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewBtcDepositRequest {
    sender: String,
    receiver: String,
}

pub async fn req(_req: HttpRequest, request_data: NewBtcDepositRequest) -> BackendRes<String> {
    //todo: 目前该接口不做限制，后续看怎么收拢权限
    let NewBtcDepositRequest { sender, receiver } = request_data;

    let airdrop_info = AirdropEntity::find(AirdropFilter::ByBtcAddress(&receiver)).await?;
    if airdrop_info.is_empty() {
        //Err(BackendError::InternalError("".to_string()))?;
        warn!("receiver {} isn't belong us", receiver);
        return Ok(None);
    }

    //directly的方式不允许重复评级，防止被覆盖
    if airdrop_info.len() == 1
        && airdrop_info[0].airdrop.btc_address.is_some()
        && airdrop_info[0].airdrop.btc_level.is_none()
    {
        let grade = query_wallet_grade(&sender).await?;
        AirdropEntity::update_single(
            AirdropUpdater::BtcLevel(grade),
            AirdropFilter::ByBtcAddress(&receiver),
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
