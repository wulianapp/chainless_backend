use actix_web::{web, HttpRequest};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::KeyRole;
use common::hash::Hash;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};

use serde::{Deserialize, Serialize};

//use log::debug;
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::judge_role_by_strategy;
use common::error_code::{AccountManagerError::*, WalletError};
use common::error_code::{BackendError, BackendRes};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager::UserInfoEntity, PsqlOp};

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

    let user_info =
        UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
            .await
            .map_err(|_e| PhoneOrEmailNotRegister)?
            .into_inner();

    let devices =
        DeviceInfoEntity::find(DeviceInfoFilter::ByDeviceUser(&device_id, &user_info.id))
            .await?;

    //todo: 通过前端签名消息来处理，签名reset_password
    let role = KeyRole::Master;
    if role != KeyRole::Master {
        Err(WalletError::UneligiableRole(role, KeyRole::Master))?;
    }
    

    //check captcha
    Captcha::check_and_delete(
        &user_info.id.to_string(),
        &captcha,
        Usage::ResetLoginPassword,
    )?;

    //modify user's password  at db
    UserInfoEntity::update_single(
        UserUpdater::LoginPwdHash(&new_password.hash(), user_info.token_version + 1),
        UserFilter::ById(&user_info.id),
    )
    .await?;

    //clear retry status after login by captcha
    let retry_storage = &mut super::login::LOGIN_RETRY
        .lock()
        .map_err(|e| BackendError::InternalError(e.to_string()))?;
    retry_storage.remove(&user_info.id);

    Ok(None::<String>)
}
