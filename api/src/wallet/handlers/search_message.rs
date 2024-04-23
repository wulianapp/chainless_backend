use actix_web::HttpRequest;
use common::data_structures::coin_transaction::CoinTransaction;
use common::data_structures::AccountMessage;
use common::utils::math::coin_amount::raw2display;

use crate::utils::token_auth;
use crate::wallet::add_servant;
use common::error_code::AccountManagerError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::device_info::*;
use models::secret_store::*;
use models::PsqlOp;
use crate::wallet::{SearchMessageResponse,CoinTransactionTmp1};

pub(crate) async fn req(req: HttpRequest) -> BackendRes<SearchMessageResponse> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let user = UserInfoView::find_single(UserFilter::ById(user_id))
        .map_err(|_e| AccountManagerError::UserIdNotExist)?;


    let mut messages = SearchMessageResponse::default();
    //if newcomer device not save,notify it to do
    let device_info = DeviceInfoView::find(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device_info.len() == 1 
    && device_info[0].device_info.hold_pubkey.is_some()
    && !device_info[0].device_info.holder_confirm_saved 
    {
        let secret = SecretStoreView::find_single(SecretFilter::ByPubkey(
            device_info[0].device_info.hold_pubkey.as_ref().unwrap(),
        ))?;
        messages.newcomer_became_sevant.push(secret.secret_store);//(AccountMessage::NewcomerBecameSevant())
    }
    

    let coin_txs = CoinTxView::find(CoinTxFilter::ByAccountPending(&user.user_info.main_account))?;
    let mut tx_msg = coin_txs
        .into_iter()
        .map(|tx| {
            CoinTransactionTmp1 {
                order_id: tx.transaction.order_id,
                tx_id: tx.transaction.tx_id,
                coin_type: tx.transaction.coin_type,
                from: tx.transaction.from,
                to: tx.transaction.to,
                amount: raw2display(tx.transaction.amount),
                expire_at: tx.transaction.expire_at,
                memo: tx.transaction.memo,
                stage: tx.transaction.stage,
                coin_tx_raw: tx.transaction.coin_tx_raw,
                chain_tx_raw: tx.transaction.chain_tx_raw,
                signatures: tx.transaction.signatures,
                tx_type: tx.transaction.tx_type,
                chain_status: tx.transaction.chain_status,
            }
        })
        .collect::<Vec<CoinTransactionTmp1>>();

    messages.coin_tx.append(&mut tx_msg);
    let uncompleted_txs = CoinTxView::find(CoinTxFilter::BySenderUncompleted(&user.user_info.main_account))?;
    if !uncompleted_txs.is_empty(){
        messages.have_uncompleted_txs = true;
    }   

    Ok(Some(messages))
}
