use actix_web::HttpRequest;
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::CoinType;
use common::data_structures::KeyRole;
use common::utils::math::coin_amount::display2raw;

use serde::Deserialize;
use serde::Serialize;
use crate::utils::get_user_context;
use crate::utils::token_auth;
use crate::wallet::handlers::*;
use common::error_code::BackendRes;
use common::error_code::WalletError;

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
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    let GenDepositSigRequest { coin, amount } = request_data;
    let amount = display2raw(&amount).map_err(|_e| WalletError::UnSupportedPrecision)?;

    let coin: CoinType = coin
        .parse()
        .map_err(|_e| BridgeError::CoinNotSupport("".to_string()))?;
    if coin == CoinType::DW20 {
        Err(BridgeError::CoinNotSupport(coin.to_string()))?
    }

    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;

    let (sig, deadline, cid) = bridge_cli
        .sign_deposit_info(coin, amount, &main_account)
        .await?;

    Ok(Some(GenDepositResponse { cid, deadline, sig }))
}
