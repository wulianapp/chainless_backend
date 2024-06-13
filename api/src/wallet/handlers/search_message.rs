use actix_web::HttpRequest;

use common::utils::math::coin_amount::raw2display;
use common::utils::time::now_millis;

use secret_store::SecretStore;

use super::*;
use crate::utils::token_auth;

use common::data_structures::{
    coin_transaction::{CoinSendStage, TxType},
    TxStatusOnChain,
};

use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::coin_transfer::{CoinTxEntity, CoinTxFilter};

use models::secret_store::*;
use models::PsqlOp;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CoinTransactionResponse {
    pub order_id: String,
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: String,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub stage: CoinSendStage,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<String>,
    pub tx_type: TxType,
    pub chain_status: TxStatusOnChain,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SearchMessageResponse {
    pub newcomer_became_sevant: Vec<SecretStore>,
    pub coin_tx: Vec<CoinTransactionResponse>,
    pub have_uncompleted_txs: bool,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<SearchMessageResponse> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let user = UserInfoEntity::find_single(UserFilter::ById(&user_id)).await?;
    if user.user_info.main_account.is_none() {
        return Ok(None);
    }
    let mut messages = SearchMessageResponse::default();
    //if newcomer device not save,notify it to do
    let device_info =
        DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, &user_id))
            .await?
            .device_info;
    if device_info.hold_pubkey.is_some() && !device_info.holder_confirm_saved {
        let secret = SecretStoreEntity::find_single(SecretFilter::ByPubkey(
            device_info.hold_pubkey.as_ref().unwrap(),
        ))
        .await?;
        messages.newcomer_became_sevant.push(secret.secret_store); //(AccountMessage::NewcomerBecameSevant())
    }

    let coin_txs = CoinTxEntity::find(CoinTxFilter::ByAccountPending(
        user.user_info.main_account.as_ref().unwrap(),
    ))
    .await?;
    let mut tx_msg = coin_txs
        .into_iter()
        .filter(|x| now_millis() <= x.transaction.expire_at)
        .map(|tx| CoinTransactionResponse {
            order_id: tx.transaction.order_id,
            tx_id: tx.transaction.tx_id,
            coin_type: tx.transaction.coin_type,
            from: tx.transaction.sender,
            to: tx.transaction.receiver,
            amount: raw2display(tx.transaction.amount),
            expire_at: tx.transaction.expire_at,
            memo: tx.transaction.memo,
            stage: tx.transaction.stage,
            coin_tx_raw: tx.transaction.coin_tx_raw,
            chain_tx_raw: tx.transaction.chain_tx_raw,
            signatures: tx.transaction.signatures,
            tx_type: tx.transaction.tx_type,
            chain_status: tx.transaction.chain_status,
        })
        .collect::<Vec<CoinTransactionResponse>>();

    messages.coin_tx.append(&mut tx_msg);

    if have_no_uncompleted_tx(&user.user_info.main_account.unwrap())
        .await
        .is_err()
    {
        messages.have_uncompleted_txs = true;
    }

    Ok(Some(messages))
}
