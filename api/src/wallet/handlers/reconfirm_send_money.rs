use actix_web::{web, HttpRequest};

use common::data_structures::wallet::CoinTxStatus;

use crate::wallet::ReconfirmSendMoneyRequest;
use common::http::{token_auth, BackendRes};
use models::coin_transfer::CoinTxFilter;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let _user_id = token_auth::validate_credentials(&req)?;

    //todo: check must be main device
    let ReconfirmSendMoneyRequest {
        device_id: _,
        tx_index,
        confirmed_sig,
    } = request_data.0;

    if let Some(sig) = confirmed_sig {
        models::coin_transfer::update_status(
            CoinTxStatus::SenderReconfirmed,
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
        //todo: broadcast
        let coin_txs = models::coin_transfer::get_transactions(CoinTxFilter::ByTxIndex(tx_index))?;
        let coin_tx = coin_txs.first().unwrap();
        //broadcast
        blockchain::general::broadcast_tx_commit_from_raw2(
            coin_tx.transaction.chain_tx_raw.as_ref().unwrap(),
            &sig,
        )
        .await;
    } else {
        models::coin_transfer::update_status(
            CoinTxStatus::SenderCanceled,
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    }
    Ok(None::<String>)
}
