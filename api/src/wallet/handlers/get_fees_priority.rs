use std::collections::HashMap;

use actix_web::HttpRequest;

use blockchain::{fees_call::FeesCall, multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf}};

use crate::utils::token_auth;
use common::{data_structures::wallet::CoinType, error_code::BackendRes};
use serde::{Deserialize, Serialize};
use common::error_code::BackendError::ChainError;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: HashMap<String,SubAccConf>,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<CoinType>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new()?;

    let fees_priority = fees_call_cli.get_fees_priority(&main_account).await?;
    Ok(Some(fees_priority))
}