use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{
    account_manager::{UserFilter, UserInfoView, UserUpdater},
    device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView},
    general::get_pg_pool_connect,
    secret_store::{SecretFilter, SecretStoreView, SecretUpdater},
    PgLocalCli, PsqlOp,
};

use crate::{
    utils::{
        captcha::{Captcha, Usage},
        token_auth,
    }
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
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    let (user_info, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user_info.main_account;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let UpdateSecurityRequest {
        anwser_indexes,
        secrets,
        captcha,
    } = request_data;
    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::SetSecurity)?;

    UserInfoView::update_single(
        UserUpdater::AnwserIndexes(&anwser_indexes),
        UserFilter::ById(user_id),
        &mut db_cli,
    )
    .await?;

    for secret in secrets {
        SecretStoreView::update_single(
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
            DeviceInfoView::update_single(
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
