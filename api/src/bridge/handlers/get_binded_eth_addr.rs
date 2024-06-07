use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::CoinType;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
//use log::debug;
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");
    let mut db_cli = get_pg_pool_connect().await?;
    let (user_id, _,device_id, _) = token_auth::validate_credentials(&req,&mut db_cli).await?;
    let (user, _current_strategy, _device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account.unwrap();

    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;

    let eth_addr = bridge_cli.get_binded_eth_addr(&main_account).await?;
    println!("eth_addr {:?} ", eth_addr);

    Ok(eth_addr)
}
