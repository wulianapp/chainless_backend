use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use common::data_structures::SecretKeyType;
use common::error_code::WalletError;
use common::error_code::{BackendRes};
use crate::utils::token_auth;
use models::account_manager::{UserFilter, UserInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::PsqlOp;
use models::secret_store::SecretStoreView;

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
    let user_info = UserInfoView::find_single(
        UserFilter::ById(user_id)
    )?.user_info;

    models::general::transaction_begin()?;
    //backup servant prikeys
    if !SecretStoreView::find(
        SecretFilter::ByPubkey(new_servant.clone())
    )?.is_empty() {
        Err(WalletError::PubkeyAlreadyExist)?
    }

    //todo: key,master_id
    let secret_info = SecretStoreView::new_with_specified(
        &new_servant, user_id, 
        "encrypted_prikey_by_password", 
        "encrypted_prikey_by_answer"
    );
    
    secret_info.insert()?;
   
    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli.get_strategy(&account_id).await.unwrap().unwrap();
    current_strategy.servant_pubkey.push(new_servant);
    multi_sig_cli.update_servant_pubkey(&account_id, 
        current_strategy.servant_pubkey).await?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
