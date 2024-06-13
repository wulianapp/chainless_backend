use actix_web::HttpRequest;
use common::data_structures::account_manager::UserInfo;

use common::data_structures::KeyRole;
use common::error_code::AccountManagerError::{self};

//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::{get_user_context, token_auth};

use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};

use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplenishContactRequest {
    contact: String,
    captcha: String,
}

pub async fn req(req: HttpRequest, request_data: ReplenishContactRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let res = account_manager::UserInfoEntity::find_single(UserFilter::ById(&user_id)).await?;
    //todo:
    //新设备或者主设备
    if res.user_info.main_account.is_some() {
        let role = get_user_context(&user_id, &device_id).await?.role()?;
        crate::wallet::handlers::check_role(role, KeyRole::Master)?;
    };

    let ReplenishContactRequest {
        contact: replenish_contact,
        captcha,
    } = request_data;
    Captcha::check_and_delete(&user_id.to_string(), &captcha, Usage::ReplenishContact)?;

    let replenish_contact_type: ContactType = replenish_contact.parse()?;

    let UserInfo {
        email,
        phone_number,
        ..
    } = res.user_info;

    if !UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&replenish_contact))
        .await?
        .is_empty()
    {
        Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
    }

    if replenish_contact_type == ContactType::Email && email.is_none() {
        UserInfoEntity::update_single(
            UserUpdater::Email(&replenish_contact),
            UserFilter::ById(&user_id),
        )
        .await?;
    } else if replenish_contact_type == ContactType::PhoneNumber && phone_number.is_none() {
        UserInfoEntity::update_single(
            UserUpdater::PhoneNumber(&replenish_contact),
            UserFilter::ById(&user_id),
        )
        .await?;
    } else {
        Err(AccountManagerError::ContactAlreadyReplenished)?;
    }

    Ok(None)
}
