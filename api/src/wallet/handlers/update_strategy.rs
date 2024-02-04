use actix_web::{HttpRequest, web};
use serde::Serialize;
use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::secret_store::SecretFilter;
use models::wallet::WalletFilter;
use crate::wallet::{AddServantRequest, DirectSendMoneyRequest, UpdateStrategy};
use blockchain::ContractClient;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: web::Json<UpdateStrategy>,
) -> BackendRes<String>{
    //todo: must be called by main device
    let user_id = token_auth::validate_credentials(&req)?;
    let UpdateStrategy {account_id,device_id, strategy} = request_data.0;

    //fixme:
    let strategy = strategy.into_iter().map(|x| MultiSigRank{
        min: x.min,
        max_eq: x.max_eq,
        sig_num: x.sig_num,
    }).collect();

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap();
    multi_sig_cli.set_strategy(&account_id,
                               current_strategy.main_device_pubkey,
                               current_strategy.servant_device_pubkey,
                               strategy
    ).await.unwrap();

    Ok(None::<String>)
}