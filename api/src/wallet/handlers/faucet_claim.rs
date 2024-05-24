use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, FaucetClaimRequest};
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
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

pub async fn req(req: HttpRequest,req_data:FaucetClaimRequest) -> BackendRes<String> {
    let user_id = token_auth::validate_credentials(&req)?;
    //let mut pg_cli = get_pg_pool_connect().await?;
    //let main_account = super::get_main_account(user_id, &mut pg_cli).await?;
    let account = req_data.account_id;
    let multi_sig_cli = ContractClient::<MultiSig>::new().await?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&account).await?;
    if master_list.len() != 1 {
        Err(BackendError::RequestParamInvalid("".to_string()))?;
    }
    let coin_list = get_support_coin_list();
    for coin in coin_list {
        let coin_cli: ContractClient<Coin> =
            ContractClient::<Coin>::new_with_type(coin.clone()).await?;
        let amount = if coin.eq(&CoinType::ETH) {
            10000000000000000
        } else {
            100000000000000000000
        };
        let _balance = coin_cli.send_coin(&account, amount).await?;
    }
    Ok(None)
}
