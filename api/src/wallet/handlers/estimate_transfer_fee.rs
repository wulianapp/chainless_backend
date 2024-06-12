use std::num::ParseIntError;

use actix_web::HttpRequest;

use blockchain::{
    coin::Coin,
    fees_call::FeesCall,
    multi_sig::{MultiSig, MultiSigRank, StrategyData},
    ContractClient,
};
use models::{
    account_manager::{UserFilter, UserInfoEntity},
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    secret_store::{SecretFilter, SecretStoreEntity},
    PsqlOp,
};
use tracing::{debug, info, warn};

use crate::utils::token_auth;
use common::{
    data_structures::secret_store::SecretStore,
    error_code::{AccountManagerError, BackendError, BackendRes},
    utils::math::*,
};
use common::{
    data_structures::CoinType,
    error_code::{parse_str, to_param_invalid_error, BackendError::ChainError, WalletError},
    utils::math::coin_amount::{display2raw, raw2display},
};
use serde::{de::value, Deserialize, Serialize};

use super::get_fees_priority;

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct EstimateTransferFeeResponse {
    pub coin: CoinType,
    pub amount: String,
    pub balance_enough: bool,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EstimateTransferFeeRequest {
    pub coin: String,
    pub amount: String,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: EstimateTransferFeeRequest,
) -> BackendRes<EstimateTransferFeeResponse> {

    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;

    let main_account = super::get_main_account(user_id).await?;

    let EstimateTransferFeeRequest { coin, amount } = request_data;
    let coin: CoinType = coin.parse().map_err(to_param_invalid_error)?;
    let amount = display2raw(&amount).map_err(|_e| WalletError::UnSupportedPrecision)?;
    let (coin, amount, balance_enough) =
        super::estimate_transfer_fee(&main_account, &coin, amount).await?;

    Ok(Some(EstimateTransferFeeResponse {
        coin,
        amount: raw2display(amount),
        balance_enough,
    }))
}
