use std::collections::BTreeMap;

use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, SubAccConf};


use crate::utils::token_auth;

use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataResponse {
    pub multi_sig_ranks: Vec<MultiSigRankResponse>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: BTreeMap<String, SubAccConf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRankResponse {
    min: String,
    max_eq: String,
    sig_num: u8,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<StrategyDataResponse> {
    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    let main_account = super::get_main_account(user_id).await?;
    let multi_cli = blockchain::ContractClient::<MultiSig>::new_query_cli().await?;

    let strategy = multi_cli.get_strategy(&main_account).await?;
    let master_pubkey: String = multi_cli.get_master_pubkey(&main_account).await?;

    Ok(strategy.map(|data| {
        //let subaccounts = data.subaccounts.iter().map(|x| x.to_string()).collect();
        let rank_external = data
            .multi_sig_ranks
            .iter()
            .map(|rank| MultiSigRankResponse {
                min: raw2display(rank.min),
                max_eq: raw2display(rank.max_eq),
                sig_num: rank.sig_num,
            })
            .collect();

        StrategyDataResponse {
            multi_sig_ranks: rank_external,
            master_pubkey,
            servant_pubkeys: data.servant_pubkeys,
            subaccounts: data.sub_confs,
        }
    }))
}
