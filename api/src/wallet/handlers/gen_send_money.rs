use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::KeyRole2;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, GenSendMoneyRequest};
use blockchain::multi_sig::{MultiSig, PubkeySignInfo};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

pub(crate) async fn req(req: HttpRequest, request_data: GenSendMoneyRequest) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;

    let (_user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let GenSendMoneyRequest { order_id } = request_data;

    let coin_tx = models::coin_transfer::CoinTxView::find_single(
        models::coin_transfer::CoinTxFilter::ByOrderId(&order_id),
    )?;
    //不好卡就先不卡了、强制转账、给子账户转账
    /****
    if coin_tx.transaction.status != CoinTxStatus::ReceiverApproved {
        Err(WalletError::TxStatusIllegal(coin_tx.transaction.status,CoinTxStatus::ReceiverApproved))?;
    }
    **/

    let servant_sigs = coin_tx
        .transaction
        .signatures
        .iter()
        .map(|data| PubkeySignInfo {
            pubkey: data[..64].to_string(),

            signature: data[64..].to_string(),
        })
        .collect();

    let cli = blockchain::ContractClient::<MultiSig>::new()?;
    let (tx_id, chain_raw_tx) = cli
        .gen_send_money_raw(
            servant_sigs,
            &coin_tx.transaction.from,
            &coin_tx.transaction.to,
            coin_tx.transaction.coin_type,
            coin_tx.transaction.amount,
            coin_tx.transaction.expire_at,
        )
        .await?;
    models::coin_transfer::CoinTxView::update_single(
        CoinTxUpdater::TxidTxRaw(&tx_id, &chain_raw_tx),
        CoinTxFilter::ByOrderId(&order_id),
    )?;
    Ok(Some(tx_id))
}
