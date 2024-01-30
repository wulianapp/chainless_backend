use actix_web::{HttpRequest, HttpResponse, Responder, web};
use log::{debug, info};
use common::error_code::{AccountManagerError, WalletError};
use common::http::{ApiRes, token_auth};
use models::account_manager;
use models::account_manager::UserFilter;
use crate::account_manager::{ResetPasswordRequest, captcha};
use crate::account_manager::captcha::{Captcha, Kind};
use common::error_code::AccountManagerError::*;
use common::error_code::ApiCommonError::*;

pub async fn req(
    request_data: web::Json<ResetPasswordRequest>,
) -> ApiRes<String> {
    debug!("start reset_password");
    let ResetPasswordRequest {
        device_id:String,
        contact,
        captcha,
        new_password,
    } = request_data.clone();
    //todo: check if is master device

    //check captcha
    Captcha::check_user_code(&contact, &captcha,Kind::reset_password)?;

    let user_at_stored = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact))
        .ok_or(PhoneOrEmailAlreadyRegister.into())?;

    //modify user's password  at db
    account_manager::update_password(&new_password, UserFilter::ById(user_at_stored.id));
    Ok(None::<String>)
}