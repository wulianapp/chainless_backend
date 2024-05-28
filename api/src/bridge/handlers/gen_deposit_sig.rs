use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::CoinType;
use common::data_structures::KeyRole2;
use common::utils::math::coin_amount::display2raw;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use serde::Deserialize;
use serde::Serialize;
//use log::debug;
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

#[derive(Deserialize, Serialize, Debug)]
pub struct GenDepositResponse {
    pub cid: u64,
    pub deadline: u64,
    pub sig: String,
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenDepositSigRequest {
    coin: String,
    amount: String,
}

pub async fn req(
    req: HttpRequest,
    request_data: GenDepositSigRequest,
) -> BackendRes<GenDepositResponse> {
    //todo: check jwt token
    debug!("start reset_password");
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;
    let (user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account;

    if main_account.eq("") {
        Err(WalletError::NotSetSecurity)?
    }

    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;

    let GenDepositSigRequest { coin, amount } = request_data.clone();
    let amount = display2raw(&amount).map_err(BackendError::RequestParamInvalid)?;

    let coin: CoinType = coin
        .parse()
        .map_err(|_e| BridgeError::CoinNotSupport("".to_string()))?;
    if coin == CoinType::DW20 {
        Err(BridgeError::CoinNotSupport(coin.to_string()))?
    }

    let bridge_cli = ContractClient::<Bridge>::new().await?;

    let (sig, deadline, cid) = bridge_cli
        .sign_deposit_info(coin, amount, &main_account)
        .await?;
    println!("sig {} ", sig);

    Ok(Some(GenDepositResponse { cid, deadline, sig }))
}
