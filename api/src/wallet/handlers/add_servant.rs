use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use common::http::{token_auth, BackendRes};
use models::secret_store::{SecretFilter, SecretUpdate};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let _user_id = token_auth::validate_credentials(&req)?;
    let AddServantRequest {
        account_id,
        device_id: _,
        secret_key_data: key_data,
        pubkey,
    } = request_data;

    //backup servant prikeys
    models::general::transaction_begin()?;
    let mut secret_info = models::secret_store::SecretStore2::find_single(SecretFilter::ByAccountId(account_id.clone()))?;
    secret_info.servant_encrypted_prikeys.push(key_data);
    models::secret_store::SecretStore2::update(
        SecretUpdate::Servant(secret_info.servant_encrypted_prikeys),
        SecretFilter::ByAccountId(account_id.clone()),
    )?;

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap();
    current_strategy.servant_device_pubkey.push(pubkey);
    multi_sig_cli
        .set_strategy(
            &account_id,
            current_strategy.main_device_pubkey,
            current_strategy.servant_device_pubkey,
            current_strategy.multi_sig_ranks,
        )
        .await
        .unwrap();

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
