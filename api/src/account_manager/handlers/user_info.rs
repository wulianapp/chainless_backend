use actix_web::HttpRequest;
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoView};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
//use super::super::ContactIsUsedRequest;
use crate::account_manager::UserInfoRequest;
use crate::utils::token_auth;

#[derive(Serialize,Deserialize, Debug)]
pub struct UserInfoTmp {
    pub id: u32,
    pub phone_number: String,
    pub email: String,
    pub sign_pwd_hash: String,
    pub is_frozen: bool,
    pub predecessor: Option<u32>,
    pub laste_predecessor_replace_time: u64,
    pub invite_code: String,
    pub main_account: String,
}

pub fn req(request: HttpRequest) -> BackendRes<UserInfoTmp> {
    let user_id = token_auth::validate_credentials(&request)?;

    let res = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    let info = UserInfoTmp {
        id: res.id,
        phone_number: res.user_info.phone_number,
        email: res.user_info.email,
        sign_pwd_hash: res.user_info.sign_pwd_hash,
        is_frozen: res.user_info.is_frozen,
        predecessor: res.user_info.predecessor,
        laste_predecessor_replace_time: res.user_info.laste_predecessor_replace_time,
        invite_code: res.user_info.invite_code,
        main_account: res.user_info.main_account,
    };
    Ok(Some(info))
}
