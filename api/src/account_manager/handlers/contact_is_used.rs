use common::http::BackendRes;

use models::{account_manager, PsqlOp};
use models::account_manager::UserFilter;
//use super::super::ContactIsUsedRequest;
use crate::account_manager::ContactIsUsedRequest;

pub fn req(request_data: ContactIsUsedRequest) -> BackendRes<bool> {
    let ContactIsUsedRequest { contact } = request_data;
    let find_res = account_manager::UserInfoView::find(UserFilter::ByPhoneOrEmail(contact))?;
    let is_used = find_res.len() == 1;
    Ok(Some(is_used))
}
