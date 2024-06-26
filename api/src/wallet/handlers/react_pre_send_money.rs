use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::KeyRole;
use common::utils::time::now_millis;

use crate::utils::{get_user_context, token_auth};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use models::coin_transfer::CoinTxEntity;

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

    let coin_tx =
        CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id))
            .await?;
    if now_millis() > coin_tx.transaction.expire_at {
        Err(WalletError::TxExpired)?;
    }
    if coin_tx.transaction.stage != CoinSendStage::SenderSigCompleted {
        Err(WalletError::TxStageIllegal(
            coin_tx.transaction.stage,
            CoinSendStage::SenderSigCompleted,
        ))?;
    }

    //message max is 10，
    if is_agreed {
        //todo:check user_id's main account_id is receiver

        let cli = blockchain::ContractClient::<MultiSig>::new_query_cli().await?;
        let servant_sigs = coin_tx
            .transaction
            .signatures
            .iter()
            .map(|data| data.parse())
            .collect::<Result<Vec<_>, BackendError>>()?;

        //todo: replace with new api(gen_chain_tx) whereby avert tx expire
        let (tx_id, chain_raw_tx) = cli
            .gen_send_money_raw(
                servant_sigs,
                &coin_tx.transaction.sender,
                &coin_tx.transaction.receiver,
                coin_tx.transaction.coin_type,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            )
            .await?;
        CoinTxEntity::update_single(
            CoinTxUpdater::ChainTxInfo(&tx_id, &chain_raw_tx, CoinSendStage::ReceiverApproved),
            CoinTxFilter::ByOrderId(&order_id),
        )
        .await?;
    } else {
        CoinTxEntity::update_single(
            CoinTxUpdater::Stage(CoinSendStage::ReceiverRejected),
            CoinTxFilter::ByOrderId(&order_id),
        )
        .await?;
    };

    Ok(None)
}
