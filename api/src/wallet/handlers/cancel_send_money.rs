use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, PubkeySignInfo};
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use crate::wallet::CancelSendMoneyRequest;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater, CoinTxView};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<CancelSendMoneyRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let _device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    let (_user,current_strategy,device) = super::get_session_state(user_id,&device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Master)?;

    //todo: check must be main device
    let CancelSendMoneyRequest {
        order_id,
    } = request_data.0;
    let tx = CoinTxView::find_single(CoinTxFilter::ByOrderId(&order_id))?;
    //todo: chain status
    if tx.transaction.stage == CoinSendStage::SenderReconfirmed{
        Err(WalletError::TxAlreadyConfirmed)?;
    }else{
        models::coin_transfer::CoinTxView::update_single(
            CoinTxUpdater::Stage(CoinSendStage::SenderCanceled),
            CoinTxFilter::ByOrderId(&order_id),
        )?;
    }
    Ok(None)
}
