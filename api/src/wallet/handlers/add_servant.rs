use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use common::http::{token_auth, BackendRes};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use common::error_code::BackendError::InternalError;
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let user_id = token_auth::validate_credentials(&req)?;
    let AddServantRequest {
        account_id,
        device_id: _,
        //secret_key_data: key_data,
        pubkey: new_servant,
    } = request_data;

    let keys = super::pending_pubkey::get_user_pending_keys(user_id)?;
    let (prikey,_) = keys
        .iter()
        .find(|(prikey,pubkey)| *pubkey == new_servant)
        .ok_or( InternalError("".to_string()))?;

    models::general::transaction_begin()?;
    //backup servant prikeys
    let mut secret_info = models::secret_store::SecretStore2::find_single(SecretFilter::ByAccountId(account_id.clone()))?;
    secret_info.servant_encrypted_prikeys.push(prikey.to_owned());
    models::secret_store::SecretStore2::update(
        SecretUpdater::Servant(secret_info.servant_encrypted_prikeys),
        SecretFilter::ByAccountId(account_id.clone()),
    )?;


    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap().unwrap();
    current_strategy.servant_device_pubkey.push(new_servant);
    multi_sig_cli.update_servant_pubkey(&account_id, 
        current_strategy.servant_device_pubkey).await?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
