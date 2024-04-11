use actix_web::{web, HttpRequest};
use blockchain::bridge::Bridge;
use blockchain::ContractClient;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::bridge::BindEthAddrRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes};
use common::error_code::{AccountManagerError::*, WalletError};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};
use crate::wallet::handlers::*;

pub async fn req(
    req: HttpRequest,
    request_data: BindEthAddrRequest,
) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user,current_strategy,device) = 
    get_session_state(user_id,&device_id).await?;
    let main_account = user.main_account;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role,KeyRole2::Master)?;

    let BindEthAddrRequest {
        eth_addr,
        user_eth_sig
    } = request_data.clone();

    let bridge_cli = ContractClient::<Bridge>::new().unwrap();

    //todo: InternalError
    if !bridge_cli.verify_eth_bind_sign(&eth_addr,&main_account,&user_eth_sig){
        Err(BackendError::InternalError("".to_string()))?;
    }

    let bind_res = bridge_cli.bind_eth_addr(
        &main_account,
    &eth_addr,
    &user_eth_sig
    ).await.unwrap();
    println!("bind_res {} ",bind_res);

    Ok(None)
}