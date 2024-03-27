use std::collections::HashMap;

use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};

use crate::utils::token_auth;
use common::error_code::BackendRes;
use serde::{Deserialize, Serialize};
use common::error_code::BackendError::ChainError;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: HashMap<String,SubAccConf>,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<StrategyDataTmp> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;

    let strategy = multi_cli.get_strategy(&main_account).await?;
    let master_pubkey: String = multi_cli.get_master_pubkey(&main_account).await?;

    Ok(strategy.map(|data| {
        //let subaccounts = data.subaccounts.iter().map(|x| x.to_string()).collect();
        StrategyDataTmp {
            multi_sig_ranks: data.multi_sig_ranks,
            master_pubkey,
            servant_pubkeys: data.servant_pubkeys,
            subaccounts:data.sub_confs,
        }
    }))
}
