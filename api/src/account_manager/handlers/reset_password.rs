use actix_web::{web, HttpRequest};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::BackendRes;
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
    } = request_data.clone();
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    //check captcha
    Captcha::check_user_code(&contact, &captcha, Usage::ResetLoginPassword)?;

    let user_at_stored =
        account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(&contact))
            .map_err(|_e| PhoneOrEmailNotRegister)?;

    //看是否设置了安全措施，之前是都可以，之后是只有主设备可以
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if user_at_stored.user_info.secruity_is_seted && device.device_info.key_role != KeyRole2::Master
    {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

    //modify user's password  at db
    account_manager::UserInfoView::update(
        UserUpdater::LoginPwdHash(&new_password),
        UserFilter::ById(user_at_stored.id),
    )?;
    Ok(None::<String>)
}
