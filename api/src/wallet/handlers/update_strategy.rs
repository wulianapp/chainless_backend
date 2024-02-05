use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};

use common::http::{token_auth, BackendRes};

use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;

pub(crate) async fn req(
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
    //it is impossible to get none
    let current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap();
    multi_sig_cli
        .set_strategy(
            &account_id,
            current_strategy.main_device_pubkey,
            current_strategy.servant_device_pubkey,
            strategy,
        )
        .await
        .unwrap();

    Ok(None::<String>)
}
