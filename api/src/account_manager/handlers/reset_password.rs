use actix_web::{web, HttpRequest};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::KeyRole;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use log::debug;
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::{judge_role_by_strategy, token_auth};
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PgLocalCli, PsqlOp};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    contact: String,
    captcha: String,
    new_password: String,
    device_id: String,
}

pub async fn req(
    _req: HttpRequest,
    request_data: web::Json<ResetPasswordRequest>,
) -> BackendRes<String> {
    debug!("start reset_password");
    let ResetPasswordRequest {
        contact,
        captcha,
        new_password,
        device_id,
    } = request_data.clone();

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let user_info = account_manager::UserInfoEntity::find_single(
        UserFilter::ByPhoneOrEmail(&contact),
        &mut db_cli,
    )
    .await
    .map_err(|_e| PhoneOrEmailNotRegister)?
    .into_inner();

    if let Some(account) = user_info.main_account {
        let devices = DeviceInfoEntity::find(
            DeviceInfoFilter::ByDeviceUser(&device_id, &user_info.id),
            &mut db_cli,
        )
        .await?;
        //针对用新设备修改
        let role = if devices.is_empty() {
            KeyRole::Undefined
        } else {
            //设置安全问答之前或者之后的主设备 才有权限改登录密码
            let cli = ContractClient::<MultiSig>::new_query_cli().await?;
            let strategy = cli.get_strategy(&account).await?;
            judge_role_by_strategy(
                strategy.as_ref(),
                devices[0].device_info.hold_pubkey.as_deref(),
            )?
        };

        if role != KeyRole::Master {
            Err(WalletError::UneligiableRole(role, KeyRole::Master))?;
        }
    }

    //check captcha
    Captcha::check_and_delete(
        &user_info.id.to_string(),
        &captcha,
        Usage::ResetLoginPassword,
    )?;

    //modify user's password  at db
    account_manager::UserInfoEntity::update_single(
        UserUpdater::LoginPwdHash(&new_password, user_info.token_version + 1),
        UserFilter::ById(&user_info.id),
        &mut db_cli,
    )
    .await?;

    //clear retry status after login by captcha
    let retry_storage = &mut super::login::LOGIN_RETRY
        .lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;
    retry_storage.remove(&user_info.id);

    Ok(None::<String>)
}
