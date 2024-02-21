use actix_web::HttpRequest;

use crate::wallet::searchMessageByAccountIdRequest;
use common::error_code::AccountManagerError;
use common::http::{token_auth, BackendRes};
use models::account_manager::{ UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<(String, Vec<CoinTxView>)>> {
    let user_id = token_auth::validate_credentials(&req)?;

    let user = UserInfoView::find_single(UserFilter::ById(user_id))
        .map_err(|e|AccountManagerError::UserIdNotExist)?;

    let message = user
        .user_info
        .account_ids
        .iter()
        .map(|acc_id| {
            let coin_txs = CoinTxView::find(CoinTxFilter::ByAccountPending(acc_id.to_string())).unwrap();
        (acc_id.to_string(), coin_txs)
        })
        .collect::<Vec<(String, Vec<CoinTxView>)>>();
    Ok(Some(message))
}

pub(crate) async fn req_by_account_id(
    req: HttpRequest,
    request_data: searchMessageByAccountIdRequest,
) -> BackendRes<Vec<CoinTxView>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let _ = UserInfoView::find_single(UserFilter::ById(user_id)).map_err(|e|AccountManagerError::UserIdNotExist)?;
    //todo: check if account_id is belong to user
    let coin_txs = CoinTxView::find(CoinTxFilter::ByAccountPending(request_data.account_id))?;

    Ok(Some(coin_txs))
}
