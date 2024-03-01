use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};

use common::error_code::{BackendRes};
use crate::utils::token_auth;
use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<UpdateStrategy>,
) -> BackendRes<String> {
    //todo: must be called by main device
    let _user_id = token_auth::validate_credentials(&req)?;
    let UpdateStrategy {
        account_id,
        device_id: _,
        strategy,
    } = request_data.0;

    //fixme:
    let strategy = strategy
        .into_iter()
        .map(|x| MultiSigRank {
            min: x.min,
            max_eq: x.max_eq,
            sig_num: x.sig_num,
        })
        .collect();

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    multi_sig_cli.update_rank(&account_id, strategy).await?;

    Ok(None::<String>)
}
