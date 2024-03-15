use actix_web::{web, HttpRequest};

use common::data_structures::wallet::CoinTxStatus;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use crate::wallet::ReconfirmSendMoneyRequest;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

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
