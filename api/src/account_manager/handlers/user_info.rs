use actix_web::HttpRequest;
use common::data_structures::OpStatus;
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoView};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
//use super::super::ContactIsUsedRequest;
use crate::account_manager::UserInfoRequest;
use crate::utils::token_auth;

/***
 * 
  pub phone_number: String,
    pub email: String,
    pub login_pwd_hash: String,
    pub sign_pwd_hash: String,
    //if is frozened,cannt operation anymore
    pub is_frozen: bool,
    pub predecessor: Option<u32>,
    pub laste_predecessor_replace_time: u64,
    //default is user_id
    pub invite_code: String,
    pub kyc_is_verified: bool,
    pub secruity_is_seted: bool,
    //last three time subaccounts creation
    pub create_subacc_time: Vec<u64>,
    //todo: convert to Option<String>
    pub main_account: String,
 * 
*/

#[derive(Serialize,Deserialize, Debug)]
pub struct UserInfoTmp {
    pub id: u32,
    pub phone_number: String,
    pub email: String,
    pub anwser_indexes: String,
    pub is_frozen: bool,
    pub predecessor: Option<u32>,
    pub laste_predecessor_replace_time: u64,
    pub invite_code: String,
    pub kyc_is_verified: bool,
    pub secruity_is_seted: bool,
    pub main_account: String,
    //pub op_status: OpStatus,
}

pub fn req(request: HttpRequest) -> BackendRes<UserInfoTmp> {
    let user_id = token_auth::validate_credentials(&request)?;

    let res = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    let info = UserInfoTmp {
        id: res.id,
        phone_number: res.user_info.phone_number,
        email: res.user_info.email,
        anwser_indexes: res.user_info.anwser_indexes,
        is_frozen: res.user_info.is_frozen,
        predecessor: res.user_info.predecessor,
        laste_predecessor_replace_time: res.user_info.laste_predecessor_replace_time,
        invite_code: res.user_info.invite_code,
        kyc_is_verified: res.user_info.kyc_is_verified,
        secruity_is_seted: res.user_info.secruity_is_seted,
        main_account: res.user_info.main_account,
        //op_status: res.user_info.op_status,
    };
    Ok(Some(info))
}
