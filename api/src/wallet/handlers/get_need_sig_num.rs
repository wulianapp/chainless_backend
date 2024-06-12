use std::collections::HashMap;

use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};
use common::data_structures::CoinType;
use models::general::get_pg_pool_connect;

use crate::utils::token_auth;
use common::error_code::BackendError::ChainError;
use common::{
    error_code::{BackendError, BackendRes, WalletError},
    utils::math::coin_amount::display2raw,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetNeedSigNumRequest {
    coin: String,
    amount: String,
}

pub(crate) async fn req(req: HttpRequest, request_data: GetNeedSigNumRequest) -> BackendRes<u8> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let GetNeedSigNumRequest { coin, amount } = request_data;

    let (_user, strategy, _device) =
        super::get_session_state(user_id, &device_id).await?;

    let coin_type: CoinType = coin
        .parse()
        .map_err(|_e| BackendError::RequestParamInvalid("coin not support".to_string()))?;
    let amount = display2raw(&amount).map_err(|_e| WalletError::UnSupportedPrecision)?;
    let need_sig_num = super::get_servant_need(&strategy.multi_sig_ranks, &coin_type, amount).await;
    Ok(Some(need_sig_num))
}
