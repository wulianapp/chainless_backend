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
    error_code::{AccountManagerError, BackendError, BackendRes},
};
use serde::{Deserialize, Serialize};
use common::error_code::BackendError::ChainError;
use  crate::wallet::SecretType;


pub(crate) async fn req(
    req: HttpRequest,
    request_data: GetSecretRequest,
) -> BackendRes<Vec<SecretStore>> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let cli = blockchain::ContractClient::<MultiSig>::new()?;
    let main_account = super::get_main_account(user_id)?;
    let GetSecretRequest {r#type,account_id} = request_data;
    match r#type {
        //如果指定则获取指定账户的key，否则获取当前设备的key(master_key,或者servant_key)
        SecretType::Single => {
            if let Some(account_id) = account_id{
                let pubkey = cli.get_master_pubkey(&account_id).await?;
                let secret = SecretStoreView::find_single(SecretFilter::ByPubkey(&pubkey))?;
                Ok(Some(vec![secret.secret_store]))
            }else {
                let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
                let pubkey = device.device_info.hold_pubkey
                .as_deref()
                .ok_or(AccountManagerError::UserNotSetSecurity)?;
                let secrete = SecretStoreView::find_single(SecretFilter::ByPubkey(pubkey))?;
                Ok(Some(vec![secrete.secret_store]))
            }
        },
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

            let mut sub_pubkeys:Vec<String> = strategy
            .sub_confs
            .into_values()
            .map(|x| {
                x.pubkey
            })
            .collect();
            keys.append(&mut sub_pubkeys);

            let mut secrets = vec![];
            for key in keys {
                let secrete = SecretStoreView::find_single(SecretFilter::ByPubkey(&key))?;
                secrets.push(secrete.secret_store);
            }
            Ok(Some(secrets))
        },
    } 
}
