use actix_web::{web, HttpRequest};

use common::data_structures::wallet::CoinTxStatus;

use crate::utils::token_auth;
use crate::wallet::ReconfirmSendMoneyRequest;
use common::error_code::BackendRes;
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let _user_id = token_auth::validate_credentials(&req)?;

    //todo: check must be main device
    let ReconfirmSendMoneyRequest {
        tx_index,
        confirmed_sig,
    } = request_data.0;

    if let Some(sig) = confirmed_sig {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Status(CoinTxStatus::SenderReconfirmed),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;

        let coin_tx =
            models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
        //broadcast
        blockchain::general::broadcast_tx_commit_from_raw2(
            coin_tx.transaction.chain_tx_raw.as_ref().unwrap(),
            &sig,
        )
        .await;
    } else {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Status(CoinTxStatus::SenderCanceled),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    }
    Ok(None::<String>)
}
