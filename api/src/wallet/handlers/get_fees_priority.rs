use actix_web::HttpRequest;
use models::general::get_pg_pool_connect;
use std::collections::BTreeMap;

use blockchain::{
    fees_call::FeesCall,
    multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf},
};

use crate::utils::token_auth;
use common::error_code::BackendError::ChainError;
use common::{data_structures::CoinType, error_code::BackendRes};
use serde::{Deserialize, Serialize};

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<CoinType>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;
    let main_account = super::get_main_account(user_id, &mut db_cli).await?;
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new_update_cli().await?;

    let fees_priority = fees_call_cli.get_fees_priority(&main_account).await?;
    Ok(Some(fees_priority))
}
