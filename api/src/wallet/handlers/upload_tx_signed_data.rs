use actix_web::{HttpRequest, web};
use serde::Serialize;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::coin_transfer::CoinTxFilter;
use models::wallet::{get_wallet, WalletFilter};
use crate::wallet::uploadTxSignatureRequest;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req)?;

    //todo: check must be main device
    let uploadTxSignatureRequest {
        device_id,
        tx_id,
        signature,
    } = request_data.0;

    //todo: validate signature

    let tx = models::coin_transfer::get_transactions(CoinTxFilter::ByTxId(tx_id.clone()))?;
    let mut signatures = tx.first().unwrap().transaction.signatures.clone();
    signatures.push(signature);
    models::coin_transfer::update_signature(signatures, CoinTxFilter::ByTxId(tx_id.clone()))?;
    //todo: collect enough signatures
    let wallet_info = get_wallet(WalletFilter::ByUserId(user_id))?;
    let wallet_info = &wallet_info.first().unwrap().wallet;

    if wallet_info.sign_strategies.len() == 1
        && *wallet_info.sign_strategies.first().unwrap() == "1-1".to_string()
    //todo: check sign strategy if ok,broadcast this tx
    {
        //broadcast(signatures)
        models::coin_transfer::update_status(CoinTxStatus::Broadcast, CoinTxFilter::ByTxId(tx_id))?;
    }
    Ok(None::<String>)
}