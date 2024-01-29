use std::fmt::Debug;
use actix_web::web;
use log::{info};
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError;
use common::http::ApiRes;
use models::{account_manager, secret_store};
use models::account_manager::{get_user, get_current_user_num, UserFilter, get_next_uid};
use models::secret_store::get_secret;
use crate::account_manager::captcha::{ContactType, Captcha, Kind};

fn register(device_id:String,
            contact:String,
            captcha:String,
            predecessor_invite_code:Option<String>,
            password:String,
            contact_type: ContactType,
            encrypted_prikey:String,
            pubkey:String
) -> ApiRes<String, AccountManagerError> {

    Captcha::check_user_code(&contact, &captcha,Kind::register)?;

    //check userinfo form db
    let user_at_stored = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact));
    if user_at_stored.is_some() {
        Err(AccountManagerError::PhoneOrEmailAlreadyRegister)?;
    }


    //todo: register multi_sig_contract account

    //store user info
    let this_user_id = get_next_uid();
    let mut user_info = UserInfo::default();
    user_info.pwd_hash = password; //todo: hash it again before store
    user_info.invite_code = this_user_id.to_string();
    match contact_type {
        ContactType::PhoneNumber => {
            user_info.phone_number = contact;
        }
        ContactType::Email => {
            user_info.email = contact;
        }
    }
    //pubkey is equal to account id when register
    user_info.account_ids.push(pubkey.clone());

    if let Some(code) = predecessor_invite_code{
         let predecessor = get_user(UserFilter::ByInviteCode(&code)).ok_or(
            AccountManagerError::InviteCodeNotExist
        )?;
        user_info.predecessor = Some(predecessor.id);
    }

    models::general::transaction_begin();
    account_manager::single_insert(&user_info).unwrap();
    let secret = SecretStore {
        account_id: pubkey,
        user_id: this_user_id,
        master_encrypted_prikey: encrypted_prikey,
        servant_encrypted_prikeys: vec![],
    };
    //todo: sql trans
    secret_store::single_insert(&secret)
        .map_err(|x|
            AccountManagerError::InvalidParameters("".to_string())
        )?;
    models::general::transaction_commit();
    info!("user {:?} register successfully", user_info);
    Ok(None::<String>)
}

pub mod by_email{
    use crate::account_manager::{captcha, RegisterByEmailRequest};
    use super::*;

    pub fn req(request_data: RegisterByEmailRequest) -> ApiRes<String, AccountManagerError> {
        let RegisterByEmailRequest {
            device_id,
            email,
            captcha,
            predecessor_invite_code,
            password,
            encrypted_prikey,
            pubkey,
        } = request_data;
        captcha::validate_email(&email)?;
        super::register(device_id, email, captcha, predecessor_invite_code, password, ContactType::Email,encrypted_prikey,pubkey)
    }
}

pub mod by_phone{
    use crate::account_manager::{captcha, RegisterByPhoneRequest};
    use super::*;

    pub fn req(request_data: RegisterByPhoneRequest) -> ApiRes<String, AccountManagerError> {
        let RegisterByPhoneRequest {
            device_id,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            encrypted_prikey,
            pubkey,
        } = request_data;
        captcha::validate_phone(&phone_number)?;
        super::register(device_id,phone_number,captcha,predecessor_invite_code,password,ContactType::PhoneNumber,encrypted_prikey,pubkey)
    }
}