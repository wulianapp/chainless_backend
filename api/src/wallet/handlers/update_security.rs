use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{
    account_manager::{UserFilter, UserInfoEntity, UserUpdater},
    device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater},
    general::get_pg_pool_connect,
    secret_store::{SecretFilter, SecretStoreEntity, SecretUpdater},
    PgLocalCli, PsqlOp,
};

use crate::utils::{
    captcha::{Captcha, Usage},
    get_user_context, token_auth,
};
use common::{
    data_structures::{secret_store::SecretStore, KeyRole},
    error_code::{BackendError, BackendRes, WalletError},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretStoreRequest {
    pub pubkey: String,
    pub encrypted_prikey_by_password: String,
    pub encrypted_prikey_by_answer: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecurityRequest {
    anwser_indexes: String,
    secrets: Vec<SecretStoreRequest>,
    captcha: String,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: UpdateSecurityRequest,
) -> BackendRes<String> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let UpdateSecurityRequest {
        anwser_indexes,
        secrets,
        captcha,
    } = request_data;
    Captcha::check_and_delete(&user_id.to_string(), &captcha, Usage::SetSecurity)?;

    UserInfoEntity::update_single(
        UserUpdater::AnwserIndexes(&anwser_indexes),
        UserFilter::ById(&user_id),
       
    )
    .await?;

    for secret in secrets {
        SecretStoreEntity::update_single(
            SecretUpdater::EncrypedPrikey(
                &secret.encrypted_prikey_by_password,
                &secret.encrypted_prikey_by_answer,
            ),
            SecretFilter::ByPubkey(&secret.pubkey),
           
        )
        .await?;

        //设备表不存子账户信息
        if current_strategy.sub_confs.get(&secret.pubkey).is_some() {
            DeviceInfoEntity::update_single(
                DeviceInfoUpdater::HolderSaved(false),
                DeviceInfoFilter::ByHoldKey(&secret.pubkey),
               
            )
            .await?;
        }
    }
    Ok(None)
}
