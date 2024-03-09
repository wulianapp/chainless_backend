use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest};
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::ContractClient;
use common::data_structures::wallet::get_support_coin_list;
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::PsqlOp;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;

pub async fn req(req: HttpRequest) -> BackendRes<Vec<(String,String)>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let account_id = user_info.user_info.main_account;
    let coin_list = get_support_coin_list();
    let mut balance_list = vec![];
    for coin in coin_list {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone());
        let balance = coin_cli.
        get_balance(&account_id)
        .await
        .unwrap()
        .unwrap_or("0".to_string());
        balance_list.push((coin.to_string(),balance));
    }
    Ok(Some(balance_list))
}
