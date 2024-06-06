use actix_web::{web, HttpRequest};

use blockchain::multi_sig::MultiSig;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::KeyRole2;
use common::utils::time::now_millis;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;

use crate::utils::{get_user_context, token_auth};
use common::error_code::{BackendError, BackendRes, WalletError};
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
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;
   
    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let role = context.role()?;

    super::check_role(role, KeyRole2::Master)?;

    let ReactPreSendMoneyRequest {
        order_id,
        is_agreed,
    } = request_data;

    let coin_tx = models::coin_transfer::CoinTxEntity::find_single(
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
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
        models::coin_transfer::CoinTxEntity::update_single(
            CoinTxUpdater::ChainTxInfo(&tx_id, &chain_raw_tx, CoinSendStage::ReceiverApproved),
            CoinTxFilter::ByOrderId(&order_id),
            &mut db_cli,
        )
        .await?;
    } else {
        models::coin_transfer::CoinTxEntity::update_single(
            CoinTxUpdater::Stage(CoinSendStage::ReceiverRejected),
            CoinTxFilter::ByOrderId(&order_id),
            &mut db_cli,
        )
        .await?;
    };

    Ok(None)
}
