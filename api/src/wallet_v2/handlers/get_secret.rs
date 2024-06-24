use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use models::{
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    secret_store::{SecretFilter, SecretStoreEntity},
    PsqlOp,
};

use crate::utils::{get_main_account, token_auth};
use common::error_code::WalletError;
use common::{
    data_structures::secret_store::SecretStore,
    error_code::{BackendError, BackendRes},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SecretType {
    Single,
    All,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetSecretRequest {
    pub r#type: SecretType,
    pub account_id: Option<String>,
}

pub type GetSecretResponse = Vec<SecretStore>;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: GetSecretRequest,
) -> BackendRes<GetSecretResponse> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let cli = blockchain::ContractClient::<MultiSig>::new_query_cli().await?;
    let main_account = get_main_account(&user_id).await?;
    let GetSecretRequest { r#type, account_id } = request_data;
    match r#type {
        //如果指定则获取指定账户的key，否则获取当前设备的key(master_key,或者servant_key)
        SecretType::Single => {
            if let Some(account_id) = account_id {
                let pubkey = cli.get_master_pubkey(&account_id).await?;
                let secret =
                    SecretStoreEntity::find_single(SecretFilter::ByPubkey(&pubkey)).await?;
                Ok(Some(vec![secret.secret_store]))
            } else {
                let device = DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser(
                    &device_id, &user_id,
                ))
                .await?;
                let pubkey = device
                    .device_info
                    .hold_pubkey
                    .as_deref()
                    .ok_or(WalletError::PubkeyNotExist)?;
                let secrete =
                    SecretStoreEntity::find_single(SecretFilter::ByPubkey(pubkey)).await?;
                Ok(Some(vec![secrete.secret_store]))
            }
        }
        //获取当前用户的所有master_key,servant_key,subaccount_key
        //且顺序固定
        SecretType::All => {
            let master_key = cli.get_master_pubkey(&main_account).await?;
            let mut keys = vec![master_key];
            let mut strategy = cli
                .get_strategy(&main_account)
                .await?
                .ok_or(BackendError::InternalError("".to_string()))?;
            keys.append(&mut strategy.servant_pubkeys);

            let mut sub_pubkeys: Vec<String> =
                strategy.sub_confs.into_values().map(|x| x.pubkey).collect();
            keys.append(&mut sub_pubkeys);

            let mut secrets = vec![];
            for key in keys {
                let secrete = SecretStoreEntity::find_single(SecretFilter::ByPubkey(&key)).await?;
                secrets.push(secrete.secret_store);
            }
            Ok(Some(secrets))
        }
    }
}
