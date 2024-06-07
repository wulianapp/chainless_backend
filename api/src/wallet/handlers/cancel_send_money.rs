use actix_web::{web, HttpRequest};

use blockchain::multi_sig::MultiSig;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::KeyRole;
use common::utils::time::now_millis;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;

use crate::utils::{get_user_context, token_auth};
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxEntity, CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMoneyRequest {
    order_id: String,
}

pub async fn req(req: HttpRequest, request_data: CancelSendMoneyRequest) -> BackendRes<String> {
    //todo:check user_id if valid
    let mut db_cli = get_pg_pool_connect().await?;
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;
    let _device = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let _ = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;

    let CancelSendMoneyRequest { order_id } = request_data;
    let tx = CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id), &mut db_cli).await?;

    //cann't cancle when status is ReceiverRejected、SenderCanceled、SenderReconfirmed and MultiSigExpired
    if tx.transaction.stage.clone() >= CoinSendStage::ReceiverRejected {
        Err(WalletError::TxStageIllegal(
            tx.transaction.stage,
            CoinSendStage::ReceiverRejected,
        ))?;
    } else {
        if now_millis() > tx.transaction.expire_at {
            Err(WalletError::TxExpired)?;
        }

        models::coin_transfer::CoinTxEntity::update_single(
            CoinTxUpdater::Stage(CoinSendStage::SenderCanceled),
            CoinTxFilter::ByOrderId(&order_id),
            &mut db_cli,
        )
        .await?;
    }
    Ok(None)
}
