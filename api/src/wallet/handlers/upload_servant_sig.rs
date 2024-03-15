use actix_web::{web, HttpRequest};

use common::data_structures::wallet::CoinTxStatus;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

use crate::wallet::uploadTxSignatureRequest;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Servant {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Servant,
        ))?;
    }

    //todo: check must be main device
    let uploadTxSignatureRequest {
        tx_index,
        signature,
    } = request_data.0;

    //todo: validate signature

    let tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    let mut signatures = tx.transaction.signatures.clone();
    signatures.push(signature);
    models::coin_transfer::CoinTxView::update(
        CoinTxUpdater::Signature(signatures),
        CoinTxFilter::ByTxIndex(tx_index),
    )?;
    //todo: collect enough signatures
    //let wallet_info = get_wallet(WalletFilter::ByUserId(user_id))?;
    //let wallet_info = &wallet_info.first().unwrap().wallet;

    //todo: checkout sig if is enough
    //first error deal with in models
    if tx.transaction.signatures.is_empty() {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Status(CoinTxStatus::SenderSigCompleted),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    }
    Ok(None::<String>)
}
