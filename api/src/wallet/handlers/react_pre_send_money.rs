use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::{KeyRole, PubkeySignInfo};
use common::utils::time::now_millis;

use crate::utils::{get_user_context, token_auth};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::CoinTxEntity;
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReactPreSendMoneyRequest {
    order_id: String,
    is_agreed: bool,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ReactPreSendMoneyRequest,
) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;

    let ReactPreSendMoneyRequest {
        order_id,
        is_agreed,
    } = request_data;

    let coin_tx = CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id)).await?;
    if now_millis() > coin_tx.transaction.expire_at {
        Err(WalletError::TxExpired)?;
    }
    if coin_tx.transaction.stage != CoinSendStage::SenderSigCompleted {
        Err(WalletError::TxStageIllegal(
            coin_tx.transaction.stage,
            CoinSendStage::SenderSigCompleted,
        ))?;
    }

    let stage = if is_agreed {
        CoinSendStage::ReceiverApproved
    } else {
        CoinSendStage::ReceiverRejected
    };
    CoinTxEntity::update_single(
        CoinTxUpdater::Stage(stage),
        CoinTxFilter::ByOrderId(&order_id),
    )
    .await?;
    Ok(None)
}
