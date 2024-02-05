use actix_web::web;
use log::debug;

use crate::account_manager::captcha::{Captcha, Usage};
use crate::account_manager::ResetPasswordRequest;
use common::error_code::AccountManagerError::*;
use common::http::BackendRes;
use models::account_manager;
use models::account_manager::UserFilter;

pub async fn req(request_data: web::Json<ResetPasswordRequest>) -> BackendRes<String> {
    debug!("start reset_password");
    let ResetPasswordRequest {
        device_id: _String,
        contact,
        captcha,
        new_password,
    } = request_data.clone();
    //todo: check if is master device

    //check captcha
    Captcha::check_user_code(&contact, &captcha, Usage::reset_password)?;

    let user_at_stored = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact))?
        .ok_or(PhoneOrEmailAlreadyRegister)?;

    //modify user's password  at db
    account_manager::update_password(&new_password, UserFilter::ById(user_at_stored.id))?;
    Ok(None::<String>)
}
