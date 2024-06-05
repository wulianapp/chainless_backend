use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use serde::{Deserialize, Serialize};
//use log::debug;
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenBindEthAddrSigRequest {
    eth_addr: String,
}

pub async fn req(req: HttpRequest, request_data: GenBindEthAddrSigRequest) -> BackendRes<String> {
    //todo: check jwt token
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account.unwrap();
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;

    let GenBindEthAddrSigRequest { eth_addr } = request_data.clone();

    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;

    let sig = bridge_cli.sign_bind_info(&main_account, &eth_addr).await?;
    println!("sig {} ", sig);

    Ok(Some(sig))
}
