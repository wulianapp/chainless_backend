use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::CoinType;
use common::data_structures::KeyRole2;
use common::utils::math::coin_amount::display2raw;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use serde::Deserialize;
use serde::Serialize;
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::bridge::GenDepositSigRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

#[derive(Deserialize, Serialize, Debug)]
pub struct GenDepositRes {
    pub cid: u64,
    pub deadline: u64,
    pub sig: String,
}

pub async fn req(
    req: HttpRequest,
    request_data: GenDepositSigRequest,
) -> BackendRes<GenDepositRes> {
    //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user, current_strategy, device) = get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;

    if main_account.eq("") {
        Err(WalletError::NotSetSecurity)?
    }

    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;

    let GenDepositSigRequest { coin, amount } = request_data.clone();
    let amount = display2raw(&amount).map_err(|err| BackendError::RequestParamInvalid(err))?;

    let coin: CoinType = coin
        .parse()
        .map_err(|_e| BackendError::InternalError("".to_string()))?;
    if coin == CoinType::CLY {
        Err(BackendError::InternalError("".to_string()))?
    }

    let bridge_cli = ContractClient::<Bridge>::new()?;

    let (sig, deadline, cid) = bridge_cli
        .sign_deposit_info(coin, amount, &main_account)
        .await?;
    println!("sig {} ", sig);

    Ok(Some(GenDepositRes { cid, deadline, sig }))
}
