use std::collections::BTreeMap;

use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};
use models::general::get_pg_pool_connect;

use crate::{utils::token_auth, wallet::MultiSigRankExternal};
use common::error_code::BackendError::ChainError;
use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRankTmp>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: BTreeMap<String, SubAccConf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRankTmp {
    min: String,
    max_eq: String,
    sig_num: u8,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<StrategyDataTmp> {
    let user_id = token_auth::validate_credentials(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;
    let main_account = super::get_main_account(user_id,&mut pg_cli).await?;
    let multi_cli = blockchain::ContractClient::<MultiSig>::new().await?;

    let strategy = multi_cli.get_strategy(&main_account).await?;
    let master_pubkey: String = multi_cli.get_master_pubkey(&main_account).await?;

    Ok(strategy.map(|data| {
        //let subaccounts = data.subaccounts.iter().map(|x| x.to_string()).collect();
        let rank_external = data
            .multi_sig_ranks
            .iter()
            .map(|rank| MultiSigRankTmp {
                min: raw2display(rank.min),
                max_eq: raw2display(rank.max_eq),
                sig_num: rank.sig_num,
            })
            .collect();

        StrategyDataTmp {
            multi_sig_ranks: rank_external,
            master_pubkey,
            servant_pubkeys: data.servant_pubkeys,
            subaccounts: data.sub_confs,
        }
    }))
}
