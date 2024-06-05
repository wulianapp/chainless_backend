use actix_web::HttpRequest;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoEntity};
use models::airdrop::{AirdropEntity, AirdropFilter};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::{account_manager, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::utils::token_auth;

const INVITE_URL:&str = "https://test1.chainless.top/download?code=";

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfoResponse {
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
    pub role: String,
    pub name: Option<String>,
    pub birth: Option<String>,
    pub invite_url: String
}

pub async fn req(req: HttpRequest) -> BackendRes<UserInfoResponse> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let _devices = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?;
    let user_info = account_manager::UserInfoEntity::find_single(UserFilter::ById(&user_id), &mut db_cli)
        .await?.into_inner();

    //todo:
    let role = if user_info.main_account.is_none(){
        KeyRole2::Undefined
    } else {
        let (_, current_strategy, device) =
            crate::wallet::handlers::get_session_state(user_id, &device_id, &mut db_cli).await?;
        let current_role =
            crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
        current_role
    };

    let airdrop_info = AirdropEntity::find_single(
        AirdropFilter::ByUserId(&user_id), &mut db_cli).await?.into_inner();
    
    let info = UserInfoResponse {
        id: user_info.id,
        phone_number: user_info.phone_number.unwrap_or("".to_string()),
        email: user_info.email.unwrap_or("".to_string()),
        anwser_indexes: user_info.anwser_indexes,
        is_frozen: user_info.is_frozen,
        predecessor: None,
        laste_predecessor_replace_time: 0,
        invite_code: airdrop_info.invite_code.clone(),
        kyc_is_verified: user_info.kyc_is_verified,
        secruity_is_seted: matches!(user_info.main_account,Some(_)),
        main_account: user_info.main_account.unwrap_or("".to_string()),
        role: role.to_string(),
        name: Some("Bob".to_string()),
        birth: Some("1993-04-01".to_string()),
        invite_url: format!("{},{}",INVITE_URL,airdrop_info.invite_code),
    };
    Ok(Some(info))
}
