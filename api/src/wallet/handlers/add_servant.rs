use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use common::data_structures::SecretKeyType;
use common::error_code::{AccountManagerError, WalletError};
use common::error_code::{BackendRes};
use models::device_info::DeviceInfoView;
use crate::utils::token_auth;
use models::account_manager::{UserFilter, UserInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::PsqlOp;
use tracing::error;
use models::secret_store::SecretStoreView;

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let user_id = token_auth::validate_credentials(&req)?;
    let AddServantRequest {
        main_account,
        servant_pubkey,
        servant_prikey_encryped_by_pwd,
        servant_prikey_encryped_by_answer,
        holder_device_id,
        holder_device_brand,
    } = request_data;

   /***
    * 
    1、secret
    2、chain
    3、device_info
    */

    models::general::transaction_begin()?;
    //backup servant prikeys
    if !SecretStoreView::find(
        SecretFilter::ByPubkey(servant_pubkey.clone())
    )?.is_empty() {
        Err(WalletError::PubkeyAlreadyExist)?
    }

    //todo: key,master_id
    let secret_info = SecretStoreView::new_with_specified(
        &servant_pubkey, 
        user_id, 
        &servant_prikey_encryped_by_pwd, 
        &servant_prikey_encryped_by_answer
    );
    secret_info.insert()?;
   
    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    error!("0001_ {}",main_account);
    let mut current_strategy = multi_sig_cli
    .get_strategy(&main_account)
    .await?.
    ok_or(WalletError::MainAccountNotExist(main_account.clone()))?;
    current_strategy.servant_pubkeys.push(servant_pubkey.clone());
    multi_sig_cli.update_servant_pubkey(&main_account, 
        current_strategy.servant_pubkeys).await?;


    let secret_info = DeviceInfoView::new_with_specified(
        &holder_device_id,
        &holder_device_brand,
                              user_id,
                              &servant_pubkey,
                              false
    );
    secret_info.insert()?;    

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
