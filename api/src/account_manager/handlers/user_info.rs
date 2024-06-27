use actix_web::HttpRequest;
use blockchain::airdrop::Airdrop;
use blockchain::ContractClient;
use common::constants::INVITE_URL;

use common::data_structures::KeyRole;
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoEntity};
use models::airdrop::{AirdropEntity, AirdropFilter};

use models::PsqlOp;
use serde::{Deserialize, Serialize};

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
    pub invite_code: Option<String>,
    pub kyc_is_verified: bool,
    pub main_account: String,
    pub role: String,
    pub name: Option<String>,
    pub birth: Option<String>,
    pub invite_url: Option<String>,
}

pub async fn req(req: HttpRequest) -> BackendRes<UserInfoResponse> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    //let user_context = get_user_context(&user_id, &device_id).await?;
    //let role = user_context.role()?;
    let role = KeyRole::Master;
    //let user_info = user_context.user_info;
    let user_info = UserInfoEntity::find_single(UserFilter::ById(&user_id)).await?.into_inner();


    let airdrop_info = AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id))
        .await?
        .into_inner();

    //仅实名领取之后才能显示邀请链接
    /***
    let cli = ContractClient::<Airdrop>::new_query_cli().await?;
    let user_airdrop_on_chain = cli.get_user(&user_info.main_account).await?;
    let invite_code = match user_airdrop_on_chain {
        Some(user) if user.create_cly != 0 =>{
            Some(airdrop_info.invite_code)
        },
        _ =>{
            None
        }
    };
    **/
    let invite_code = None;

    let invite_url = if let Some(ref code) = invite_code {
        let url = format!("{}{}", INVITE_URL, code);
        Some(url)
    } else {
        None
    };

    let info = UserInfoResponse {
        id: user_info.id,
        phone_number: user_info.phone_number.unwrap_or("".to_string()),
        email: user_info.email.unwrap_or("".to_string()),
        anwser_indexes: user_info.anwser_indexes,
        is_frozen: user_info.is_frozen,
        predecessor: airdrop_info.predecessor_user_id,
        laste_predecessor_replace_time: 0,
        invite_code,
        kyc_is_verified: user_info.kyc_is_verified,
        main_account: user_info.main_account,
        role: role.to_string(),
        name: Some("Bob".to_string()),
        birth: Some("1993-04-01".to_string()),
        invite_url,
    };
    Ok(Some(info))
}
