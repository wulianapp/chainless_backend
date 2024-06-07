use actix_web::HttpRequest;
use common::constants::INVITE_URL;
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
use crate::utils::{get_user_context, token_auth};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfoResponse {
    pub id: u32,
    pub phone_number: String,
    pub email: String,
    pub anwser_indexes: String,
    pub is_frozen: bool,
    pub predecessor: u32,
    pub laste_predecessor_replace_time: u64,
    pub invite_code: String,
    pub kyc_is_verified: bool,
    pub secruity_is_seted: bool,
    pub main_account: String,
    pub role: String,
    pub name: Option<String>,
    pub birth: Option<String>,
    pub invite_url: String,
}

pub async fn req(req: HttpRequest) -> BackendRes<UserInfoResponse> {
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let (user_id, _,device_id,_) = token_auth::validate_credentials(&req,&mut db_cli).await?;

    let user_context = get_user_context(&user_id,&device_id,&mut db_cli).await?;
    let role = user_context.role()?;
    let user_info = user_context.user_info;

    let airdrop_info = AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id), &mut db_cli)
        .await?
        .into_inner();

    let info = UserInfoResponse {
        id: user_info.id,
        phone_number: user_info.phone_number.unwrap_or("".to_string()),
        email: user_info.email.unwrap_or("".to_string()),
        anwser_indexes: user_info.anwser_indexes,
        is_frozen: user_info.is_frozen,
        predecessor: airdrop_info.predecessor_user_id,
        laste_predecessor_replace_time: 0,
        invite_code: airdrop_info.invite_code.clone(),
        kyc_is_verified: user_info.kyc_is_verified,
        secruity_is_seted: user_info.main_account.is_some(),
        main_account: user_info.main_account.unwrap_or("".to_string()),
        role: role.to_string(),
        name: Some("Bob".to_string()),
        birth: Some("1993-04-01".to_string()),
        invite_url: format!("{}{}", INVITE_URL, airdrop_info.invite_code),
    };
    Ok(Some(info))
}
