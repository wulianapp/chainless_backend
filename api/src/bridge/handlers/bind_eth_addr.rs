use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::get_pg_pool_connect;
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::bridge::BindEthAddrRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

pub async fn req(req: HttpRequest, request_data: BindEthAddrRequest) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let (user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
    let main_account = user.main_account;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;

    let BindEthAddrRequest {
        eth_addr,
        user_eth_sig,
    } = request_data.clone();

    let bridge_cli = ContractClient::<Bridge>::new().await?;

    //todo: 应该校验用户的签名而不是自己的
    /***
    if !bridge_cli.verify_eth_bind_sign(&eth_addr,&main_account,&user_eth_sig){
        Err(BackendError::InternalError("".to_string()))?;
    }
    **/

    let bind_res = bridge_cli
        .bind_eth_addr(&main_account, &eth_addr, &user_eth_sig)
        .await?;
    println!("bind_res {} ", bind_res);

    Ok(None)
}
