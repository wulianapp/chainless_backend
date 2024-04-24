use actix_web::{web, HttpRequest};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tokio::time::error::Elapsed;
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes};
use common::error_code::{AccountManagerError::*, WalletError};
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ResetPasswordRequest>,
) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");
    let ResetPasswordRequest {
        contact,
        captcha,
        new_password,
        device_id,
    } = request_data.clone();

    let user_at_stored =
    account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))
        .map_err(|_e| PhoneOrEmailNotRegister)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_at_stored.id))?;
   
    if user_at_stored.user_info.secruity_is_seted{
        //目前没有需要必须登陆才能改密码的需求
        /*** 
        let (token_user_id, token_device_id, _) = token_auth::validate_credentials2(&req)?;
        if user_at_stored.id != token_user_id || device_id != token_device_id {
            Err(BackendError::RequestParamInvalid("".to_string()))?;
        }
        ***/

        //看是否设置了安全措施，之前是都可以，之后是只有主设备可以
        if device.device_info.key_role != KeyRole2::Master
        {
            Err(WalletError::UneligiableRole(
                device.device_info.key_role,
                KeyRole2::Master,
            ))?;
        }
    }

    //check captcha
    Captcha::check_user_code(&user_at_stored.id.to_string(), &captcha, Usage::ResetLoginPassword)?;

    //modify user's password  at db
    account_manager::UserInfoView::update_single(
        UserUpdater::LoginPwdHash(&new_password),
        UserFilter::ById(user_at_stored.id),
    )?;
    Ok(None::<String>)
}
