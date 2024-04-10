use actix_web::{web, HttpRequest};
use blockchain::bridge::Bridge;
use blockchain::ContractClient;
use common::data_structures::wallet::CoinType;
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::bridge::GenDepositSigRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes};
use common::error_code::{AccountManagerError::*, WalletError};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};
use crate::wallet::handlers::*;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
  //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user,current_strategy,device) = 
    get_session_state(user_id,&device_id).await?;
    let main_account = user.main_account;

    let bridge_cli = ContractClient::<Bridge>::new().unwrap();

    let eth_addr = bridge_cli.get_binded_eth_addr(
        &main_account 
    ).await?;
    println!("eth_addr {:?} ",eth_addr);

    Ok(eth_addr)
}
