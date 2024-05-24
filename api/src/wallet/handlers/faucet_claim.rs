use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, get_support_coin_list_without_cly, CoinType};
use common::error_code::BackendError;
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::general::get_pg_pool_connect;
use models::PsqlOp;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let user_id = token_auth::validate_credentials(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;
    let main_account = super::get_main_account(user_id,&mut pg_cli).await?;
    let coin_list = get_support_coin_list();
    for coin in coin_list {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new_with_type(coin.clone()).await?;
        let amount = if coin.eq(&CoinType::ETH) {
            10000000000000000
        } else {
            100000000000000000000
        };
        let _balance = coin_cli.send_coin(&main_account, amount).await?;
    }
    Ok(None)
}
