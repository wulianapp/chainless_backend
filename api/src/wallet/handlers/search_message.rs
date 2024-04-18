use actix_web::HttpRequest;
use common::data_structures::wallet::CoinTransaction;
use common::data_structures::wallet::CoinTransaction2;
use common::utils::math::coin_amount::raw2display;

use crate::utils::token_auth;
use crate::wallet::add_servant;
use common::data_structures::wallet::AccountMessage;
use common::error_code::AccountManagerError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::device_info::*;
use models::secret_store::*;
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<AccountMessage> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let user = UserInfoView::find_single(UserFilter::ById(user_id))
        .map_err(|_e| AccountManagerError::UserIdNotExist)?;

    let mut messages: Vec<AccountMessage> = vec![];

    //if newcomer device not save,notify it to do
    /***
    let device_info = DeviceInfoView::find(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device_info.len() == 1 && !device_info[0].device_info.holder_confirm_saved {
        let secret = SecretStoreView::find_single(SecretFilter::ByPubkey(
            device_info[0].device_info.hold_pubkey.as_ref().unwrap(),
        ))?;
        messages.push(AccountMessage::NewcomerBecameSevant(secret.secret_store))
    }
    **/
    let mut messages = AccountMessage{
        newcomer_became_sevant: vec![],
        coin_tx: vec![]
    };

    let coin_txs = CoinTxView::find(CoinTxFilter::ByAccountPending(&user.user_info.main_account))?;
    let mut tx_msg = coin_txs
        .into_iter()
        .map(|tx| {
            let CoinTransaction { tx_id, coin_type, from, to, amount, expire_at, memo, status, coin_tx_raw, chain_tx_raw, signatures, tx_type, reserved_field1, reserved_field2, reserved_field3 } = tx.transaction;
            let transaction =  CoinTransaction2 {
                tx_index: tx.tx_index,
                tx_id,
                coin_type,
                from,
                to,
                amount: raw2display(amount),
                expire_at,
                memo,
                status,
                coin_tx_raw,
                chain_tx_raw,
                signatures,
                tx_type,
                reserved_field1,
                reserved_field2,
                reserved_field3,
            };
            transaction
        })
        .collect::<Vec<CoinTransaction2>>();

    messages.coin_tx.append(&mut tx_msg);

    Ok(Some(messages))
}
