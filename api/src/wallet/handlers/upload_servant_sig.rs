use actix_web::{web, HttpRequest};

use common::data_structures::wallet::CoinTxStatus;

use common::http::{token_auth, BackendRes};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdate};
use models::PsqlOp;

use crate::wallet::uploadTxSignatureRequest;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let _user_id = token_auth::validate_credentials(&req)?;

    //todo: check must be main device
    let uploadTxSignatureRequest {
        device_id: _,
        tx_index,
        signature,
    } = request_data.0;

    //todo: validate signature

    let tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    let mut signatures = tx.transaction.signatures.clone();
    signatures.push(signature);
    models::coin_transfer::CoinTxView::update(CoinTxUpdate::Signature(signatures),CoinTxFilter::ByTxIndex(tx_index))?;
    //todo: collect enough signatures
    //let wallet_info = get_wallet(WalletFilter::ByUserId(user_id))?;
    //let wallet_info = &wallet_info.first().unwrap().wallet;

    //todo: checkout sig if is enough
    //first error deal with in models
    if tx.transaction.signatures.len() == 0 {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdate::Status(CoinTxStatus::SenderSigCompleted),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    }
    Ok(None::<String>)
}
