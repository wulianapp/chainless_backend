use actix_web::{HttpRequest, web};
use serde::Serialize;
use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet::CoinTxStatus;
use common::error_code::{BackendError::*, WalletError::*};
use common::http::{BackendRes, token_auth};
use models::secret_store::SecretFilter;
use models::wallet::WalletFilter;
use crate::wallet::{AddServantRequest, DirectSendMoneyRequest};
use blockchain::ContractClient;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: AddServantRequest,
) -> BackendRes<String>{
    //todo: must be called by main device
    let user_id = token_auth::validate_credentials(&req)?;
    let AddServantRequest{account_id,device_id, secret_key_data: key_data ,pubkey} = request_data;

    //backup servant prikeys
    models::general::transaction_begin()?;
    let all_secret = models::secret_store::get_secret(SecretFilter::ByAccountId(&account_id))?;
    let mut current_servant_secret = all_secret.first().unwrap().secret_store.servant_encrypted_prikeys.to_vec();
    current_servant_secret.push(key_data);
    models::secret_store::update_servant(current_servant_secret,SecretFilter::ByAccountId(&account_id))?;

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap();
    current_strategy.servant_device_pubkey.push(pubkey);
    multi_sig_cli.set_strategy(&account_id,
                               current_strategy.main_device_pubkey,
                               current_strategy.servant_device_pubkey,
                               current_strategy.multi_sig_ranks
    ).await.unwrap();

    models::general::transaction_commit()?;
    Ok(None::<String>)
}