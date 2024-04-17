use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::ContractClient;
use common::data_structures::wallet::{get_support_coin_list, get_support_coin_list_without_cly};
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::PsqlOp;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use common::error_code::BackendError::ChainError;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let coin_list = get_support_coin_list_without_cly();
    for coin in coin_list {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
        let _balance = coin_cli.send_coin(&main_account, 100000000000000000000).await?;
    }
    Ok(None)
}
