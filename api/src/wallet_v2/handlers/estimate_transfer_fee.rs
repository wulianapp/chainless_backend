use actix_web::HttpRequest;
use crate::utils::{get_main_account, token_auth};
use common::error_code::BackendRes;
use common::{
    data_structures::CoinType,
    error_code::{to_param_invalid_error, WalletError},
    utils::math::coin_amount::{display2raw, raw2display},
};
use serde::{Deserialize, Serialize};

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

    let main_account = get_main_account(&user_id).await?;

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
