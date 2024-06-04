use actix_web::HttpRequest;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::data_structures::KeyRole2;
use common::error_code::AccountManagerError::{self, *};
use models::airdrop::{AirdropEntity, AirdropFilter};
use models::device_info::DeviceInfoEntity;
//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::error_code::BackendRes;
use models::account_manager::{get_next_uid, UserFilter, UserInfoEntity, UserUpdater};
use models::general::*;
use models::secret_store::SecretStoreEntity;
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplenishContactRequest {
    contact: String,
    captcha: String
}


pub async fn req(
    req: HttpRequest, request_data: ReplenishContactRequest
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let res = account_manager::UserInfoEntity::find_single(UserFilter::ById(user_id), &mut db_cli)
    .await?;
    //todo:
    //新设备或者主设备
    if res.user_info.main_account.ne("") {
        let (_, current_strategy, device) =
        crate::wallet::handlers::get_session_state(user_id, &device_id, &mut db_cli).await?;
        let current_role =  crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
        crate::wallet::handlers::check_role(current_role, KeyRole2::Master)?;
    };

    let ReplenishContactRequest {contact: replenish_contact,captcha} = request_data;
    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::ReplenishContact)?;

    let replenish_contact_type: ContactType = replenish_contact.parse()?;

    let UserInfo{email,phone_number,..} = res.user_info;

    if  replenish_contact_type == ContactType::Email && email == "".to_string() {
            UserInfoEntity::update_single(
                UserUpdater::Email(&replenish_contact), 
                UserFilter::ById(user_id), 
                &mut db_cli
            ).await?;
    }else if replenish_contact_type == ContactType::PhoneNumber && phone_number == "".to_string() {
        UserInfoEntity::update_single(
            UserUpdater::PhoneNumber(&replenish_contact), 
            UserFilter::ById(user_id), 
            &mut db_cli
        ).await?;
    }else{
        Err(AccountManagerError::ContactAlreadyReplenished)?;
    }

    Ok(None)
}