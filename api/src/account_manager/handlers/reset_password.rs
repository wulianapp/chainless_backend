use actix_web::web;
//use log::debug;
use tracing::debug;

use crate::account_manager::ResetPasswordRequest;
use crate::utils::captcha::{Captcha, Usage};
use common::error_code::AccountManagerError::*;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserUpdater};
use models::{account_manager, PsqlOp};

pub async fn req(request_data: web::Json<ResetPasswordRequest>) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");
    let ResetPasswordRequest {
        device_id: _,
        contact,
        captcha,
        new_password,
    } = request_data.clone();
    //todo: check if is master device

    //check captcha
    Captcha::check_user_code(&contact, &captcha, Usage::ResetPassword)?;

    let user_at_stored =
        account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(contact))
            .map_err(|_e| PhoneOrEmailNotRegister)?;
    //modify user's password  at db
    account_manager::UserInfoView::update(
        UserUpdater::LoginPwdHash(new_password),
        UserFilter::ById(user_at_stored.id),
    )?;
    Ok(None::<String>)
}
