use actix_web::HttpRequest;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::account_manager::UserInfoRequest;
use crate::utils::token_auth;

#[derive(Serialize, Deserialize, Debug)]
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
    pub role:String,
    pub name:Option<String>,
    pub birth:Option<String>,

    //pub op_status: OpStatus,
}

pub async fn req(request: HttpRequest) -> BackendRes<UserInfoTmp> {
    let (user_id,device_id,_) = token_auth::validate_credentials2(&request)?;
    let _devices = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id,user_id))?;
    let res = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    
    //todo:
    let role = if res.user_info.main_account.is_empty(){
        KeyRole2::Undefined
    }else{
        let (_,current_strategy,device) = 
        crate::wallet::handlers::get_session_state(user_id,&device_id).await?;
        let current_role = crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
        current_role
    };



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
        role: role.to_string(),
        name: Some("Bob".to_string()),
        birth: Some("1993-04-01".to_string())

    };
    Ok(Some(info))
}
