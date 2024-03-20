use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{
    account_manager::{UserFilter, UserInfoView},
    device_info::{DeviceInfoFilter, DeviceInfoView},
    secret_store::{SecretFilter, SecretStoreView},
    PsqlOp,
};

use crate::{utils::token_auth, wallet::GetSecretRequest};
use common::{
    data_structures::secret_store::SecretStore,
    error_code::{BackendError, BackendRes},
};
use serde::{Deserialize, Serialize};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: GetSecretRequest,
) -> BackendRes<Vec<SecretStore>> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let cli = blockchain::ContractClient::<MultiSig>::new();
    let main_account = super::get_main_account(user_id)?;
    //
    match request_data.r#type {
        crate::wallet::SecretType::CurrentDevice => {
            let pubkey =
                DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?
                    .device_info
                    .hold_pubkey
                    .ok_or(BackendError::InternalError(
                        "this haven't be servant yet".to_string(),
                    ))?;

            let secret = SecretStoreView::find_single(SecretFilter::ByPubkey(&pubkey))?;
            Ok(Some(vec![secret.secret_store]))
        }
        crate::wallet::SecretType::Master => {
            let master_key = cli.get_master_pubkey(&main_account).await;
            let secret = SecretStoreView::find_single(SecretFilter::ByPubkey(&master_key))?;
            Ok(Some(vec![secret.secret_store]))
        }
        crate::wallet::SecretType::All => {
            let master_key = cli.get_master_pubkey(&main_account).await;
            let mut keys = vec![master_key];
            let mut strategy = cli
                .get_strategy(&main_account)
                .await?
                .ok_or(BackendError::InternalError("".to_string()))?;
            keys.append(&mut strategy.servant_pubkeys);
            let mut secrets = vec![];
            for key in keys {
                let secrete = SecretStoreView::find_single(SecretFilter::ByPubkey(&key))?;
                secrets.push(secrete.secret_store);
            }
            Ok(Some(secrets))
        }
    }
}
