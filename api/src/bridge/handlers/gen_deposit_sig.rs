use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::wallet::CoinType;
use common::data_structures::KeyRole2;
use common::utils::math::coin_amount::display2raw;
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

pub async fn req(
    req: HttpRequest,
    request_data: GenDepositSigRequest,
) -> BackendRes<(String,u64)> {
    //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user,current_strategy,device) = 
    get_session_state(user_id,&device_id).await?;
    let main_account = user.main_account;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role,KeyRole2::Master)?;

    let GenDepositSigRequest {
        coin,
        amount,
        eth_depositor
    } = request_data.clone();
    let amount = display2raw(&amount)
    .map_err(|err| BackendError::RequestParamInvalid(err))?;

    let coin: CoinType =  coin.parse().map_err(|_e| BackendError::InternalError("".to_string()))?;
    if coin == CoinType::CLY{
        Err(BackendError::InternalError("".to_string()))?
    }

    let bridge_cli = ContractClient::<Bridge>::new().unwrap();

    let (sig,deadline) = bridge_cli.sign_deposit_info(
        &eth_depositor,
        coin,
        amount,
        &main_account
    ).await?;
    println!("sig {} ",sig);

    Ok(Some((sig,deadline)))
}
