use std::num::ParseIntError;

use actix_web::HttpRequest;

use blockchain::{
    coin::Coin,
    fees_call::FeesCall,
    multi_sig::{MultiSig, MultiSigRank, StrategyData},
    ContractClient,
};
use models::{
    account_manager::{UserFilter, UserInfoView}, device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, secret_store::{SecretFilter, SecretStoreView}, PsqlOp
};
use tracing::{debug, info, warn};

use crate::wallet::{EstimateTransferFeeResponse, SecretType};
use crate::{utils::token_auth, wallet::EstimateTransferFeeRequest};
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

pub(crate) async fn req(
    req: HttpRequest,
    request_data: EstimateTransferFeeRequest,
) -> BackendRes<EstimateTransferFeeResponse> {
    let (user_id, _device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let main_account = super::get_main_account(user_id,&mut pg_cli).await?;

    let EstimateTransferFeeRequest { coin, amount } = request_data;
    let coin: CoinType = coin.parse().map_err(to_param_invalid_error)?;
    let amount = display2raw(&amount)?;
    let (coin, amount, balance_enough) =
        super::estimate_transfer_fee(&main_account, &coin, amount).await?;

    Ok(Some(EstimateTransferFeeResponse {
        coin,
        amount: raw2display(amount),
        balance_enough,
    }))
}
