use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::{Responder, web};
use actix_web::error::JsonPayloadError::OverflowKnownLength;
use serde::Serialize;
use common::error_code::AccountManagerError;
use common::http::{ApiRes, token_auth};
use common::utils::time::{DAY1, MINUTE30, now_millis};
use models::account_manager;
use models::account_manager::UserFilter;
//use super::super::ContactIsUsedRequest;
use crate::account_manager::ContactIsUsedRequest;

pub fn req(request_data: ContactIsUsedRequest) -> ApiRes<bool, AccountManagerError> {
    let ContactIsUsedRequest {
        contact,
    } = request_data;
    let is_used = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact)).is_some();
    Ok(Some(is_used))
}