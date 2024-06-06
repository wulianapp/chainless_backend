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
    captcha::{Captcha, Usage}, get_user_context, token_auth
};
use common::{
    data_structures::{secret_store::SecretStore, KeyRole2},
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
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (main_account,current_strategy) = context.account_strategy()?;
    let role = context.role()?;
    
    super::check_role(role, KeyRole2::Master)?;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;


    let UpdateSecurityRequest {
        anwser_indexes,
        secrets,
        captcha,
    } = request_data;
    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::SetSecurity)?;

    UserInfoEntity::update_single(
        UserUpdater::AnwserIndexes(&anwser_indexes),
        UserFilter::ById(&user_id),
        &mut db_cli,
    )
    .await?;

    for secret in secrets {
        SecretStoreEntity::update_single(
            SecretUpdater::EncrypedPrikey(
                &secret.encrypted_prikey_by_password,
                &secret.encrypted_prikey_by_answer,
            ),
            SecretFilter::ByPubkey(&secret.pubkey),
            &mut db_cli,
        )
        .await?;

        //设备表不存子账户信息
        if current_strategy.sub_confs.get(&secret.pubkey).is_some() {
            DeviceInfoEntity::update_single(
                DeviceInfoUpdater::HolderSaved(false),
                DeviceInfoFilter::ByHoldKey(&secret.pubkey),
                &mut db_cli,
            )
            .await?;
        }
    }
    db_cli.commit().await?;
    Ok(None)
}
