use common::error_code::{AccountManagerError::*, BackendError};
use common::hash::{Hash};
use common::utils::math::random_num;
use models::airdrop::{AirdropEntity, AirdropFilter};
use models::device_info::DeviceInfoEntity;
//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Usage};

use common::error_code::BackendRes;
use models::account_manager::{UserFilter};

use models::{account_manager::UserInfoEntity, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterByPhoneRequest {
    device_id: String,
    device_brand: String,
    phone_number: String,
    captcha: String,
    password: String,
    predecessor_invite_code: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterByEmailRequest {
    device_id: String,
    device_brand: String,
    email: String,
    captcha: String,
    password: String,
    //第一个账户肯定没有predecessor
    predecessor_invite_code: String,
}

//生成十位随机数作为user_id
const MAX_RETRY_TIMES: u8 = 10;
async fn gen_user_id() -> Result<u32, BackendError> {
    for _ in 0..MAX_RETRY_TIMES {
        let num = (random_num() % 9_000_000_000 + 1_000_000_000) as u32;
        if UserInfoEntity::find(UserFilter::ById(&num))
            .await?
            .is_empty()
        {
            return Ok(num);
        } else {
            warn!("user_id {} already exist", num);
            continue;
        }
    }
    Err(BackendError::InternalError("".to_string()))
}

async fn register(
    device_id: String,
    device_brand: String,
    contact: String,
    captcha: String,
    predecessor_invite_code: String,
    password: String,
    contact_type: ContactType,
) -> BackendRes<String> {
    //check userinfo form db
    let find_res =
        UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact)).await?;
    if !find_res.is_empty() {
        Err(PhoneOrEmailAlreadyRegister)?;
    }

    //store user info
    Captcha::check_and_delete(&contact, &captcha, Usage::Register)?;

    let this_user_id = gen_user_id().await?;
    let mut view = UserInfoEntity::new_with_specified(this_user_id, &password.hash());
    match contact_type {
        ContactType::PhoneNumber => {
            view.user_info.phone_number = Some(contact.clone());
        }
        ContactType::Email => {
            view.user_info.email = Some(contact.clone());
        }
    }
    let token_version = view.user_info.token_version;
    view.insert().await?;

    //register airdrop
    let predecessor_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByInviteCode(&predecessor_invite_code))
            .await
            .map_err(|_e| InviteCodeNotExist)?;

    let predecessor_userinfo_id = predecessor_airdrop.airdrop.user_id;
    let predecessor_info = UserInfoEntity::find_single(UserFilter::ById(&predecessor_userinfo_id))
        .await?
        .into_inner();

    if let Some(main_account) = predecessor_info.main_account {
        let user_airdrop =
            AirdropEntity::new_with_specified(this_user_id, predecessor_info.id, &main_account);
        user_airdrop.insert().await?;
    } else {
        Err(PredecessorNotSetSecurity)?;
    }

    let device = DeviceInfoEntity::new_with_specified(&device_id, &device_brand, this_user_id);
    device.insert().await?;

    let token = crate::utils::token_auth::create_jwt(
        this_user_id,
        token_version,
        &device_id,
        &device_brand,
    )?;
    info!("user {} register successfully", contact);
    Ok(Some(token))
}

pub mod by_email {
    use super::*;

    pub async fn req(request_data: RegisterByEmailRequest) -> BackendRes<String> {
        let RegisterByEmailRequest {
            device_id,
            device_brand,
            email,
            captcha,
            predecessor_invite_code,
            password,
        } = request_data;
        //captcha::validate_email(&email)?;
        super::register(
            device_id,
            device_brand,
            email,
            captcha,
            predecessor_invite_code,
            password,
            ContactType::Email,
        )
        .await
    }
}

pub mod by_phone {
    use super::*;

    pub async fn req(request_data: RegisterByPhoneRequest) -> BackendRes<String> {
        let RegisterByPhoneRequest {
            device_id,
            device_brand,
            phone_number,
            captcha,
            predecessor_invite_code,
            password
        } = request_data;
        super::register(
            device_id,
            device_brand,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            ContactType::PhoneNumber,
        )
        .await
    }
}
