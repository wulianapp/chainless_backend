use common::http::BackendRes;

use models::account_manager;
use models::account_manager::UserFilter;
//use super::super::ContactIsUsedRequest;
use crate::account_manager::ContactIsUsedRequest;

pub fn req(request_data: ContactIsUsedRequest) -> BackendRes<bool> {
    let ContactIsUsedRequest { contact } = request_data;
    let is_used = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact))?.is_some();
    Ok(Some(is_used))
}
