use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};

use common::error_code::{BackendRes};
use serde::{Deserialize, Serialize};
use crate::utils::token_auth;


use crate::wallet::{getStrategyRequest};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: Vec<String>,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: getStrategyRequest,
) -> BackendRes<StrategyDataTmp> {
    let _user_id = token_auth::validate_credentials(&req)?;

    let multi_cli = blockchain::ContractClient::<MultiSig>::new();

    let strategy = multi_cli
        .get_strategy(&request_data.account_id)
        .await?;
    let master_pubkey: String = multi_cli.get_master_pubkey(&request_data.account_id).await;
    Ok(strategy.map(|data| {
        let subaccounts = data.subaccounts.iter().map(|x| x.to_string()).collect();
        StrategyDataTmp {
            multi_sig_ranks: data.multi_sig_ranks,
            master_pubkey,
            servant_pubkeys: data.servant_pubkeys,
            subaccounts,
        }
    }))
}
