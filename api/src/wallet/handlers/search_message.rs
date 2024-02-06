use actix_web::HttpRequest;

use crate::wallet::searchMessageByAccountIdRequest;
use common::error_code::AccountManagerError;
use common::http::{token_auth, BackendRes};
use models::account_manager::{get_user, UserFilter};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<(String, Vec<CoinTxView>)>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let predecessor =
        get_user(UserFilter::ById(user_id))?.ok_or(AccountManagerError::UserIdNotExist)?;

    println!("searchMessage user_id {}", user_id);
    let message = predecessor
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
    let _predecessor =
        get_user(UserFilter::ById(user_id))?.ok_or(AccountManagerError::UserIdNotExist)?;

    //todo: check if account_id is belong to user
    println!("searchMessage user_id {}", user_id);
    let coin_txs = CoinTxView::find(CoinTxFilter::ByAccountPending(request_data.account_id))?;

    Ok(Some(coin_txs))
}
