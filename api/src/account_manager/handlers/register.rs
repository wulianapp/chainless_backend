use std::fmt::Debug;
use actix_web::web;
use log::{info};
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::*;
use common::error_code::BackendError::*;
use common::http::BackendRes;
use models::{account_manager, secret_store};
use models::account_manager::{get_user, get_current_user_num, UserFilter, get_next_uid};
use crate::account_manager::captcha::{ContactType, Captcha, Usage};
use blockchain::ContractClient;
use blockchain::multi_sig::{MultiSig, MultiSigRank};


async fn register(device_id:String,
            contact:String,
            captcha:String,
            predecessor_invite_code:Option<String>,
            password:String,
            contact_type: ContactType,
            encrypted_prikey:String,
            pubkey:String
) -> BackendRes<String> {

    Captcha::check_user_code(&contact, &captcha, Usage::register)?;

    //check userinfo form db
    if let Some(_) = account_manager::get_user(UserFilter::ByPhoneOrEmail(&contact))?{
        Err(PhoneOrEmailAlreadyRegister)?;
    }


    //todo: register multi_sig_contract account

    //store user info
    let this_user_id = get_next_uid()?;
    println!("this_user_id _______{}",this_user_id);
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
         let predecessor = get_user(UserFilter::ByInviteCode(&code))?.ok_or(
            InviteCodeNotExist
        )?;
        user_info.predecessor = Some(predecessor.id);
    }

    models::general::transaction_begin()?;
    account_manager::single_insert(&user_info)?;
    let secret = SecretStore {
        account_id: pubkey.clone(),
        user_id: this_user_id,
        master_encrypted_prikey: encrypted_prikey,
        servant_encrypted_prikeys: vec![],
    };
    //todo: sql trans
    secret_store::single_insert(&secret)?;
    let multi_cli = ContractClient::<MultiSig>::new();


    //multi_cli.set_strategy(&pubkey,pubkey.clone(),vec![],vec![MultiSigRank::default()]).await.unwrap();
    multi_cli.init_strategy(&pubkey,pubkey.clone()).await.unwrap();
    models::general::transaction_commit()?;
    info!("user {:?} register successfully", user_info);
    Ok(None::<String>)
}

pub mod by_email{
    use crate::account_manager::{captcha, RegisterByEmailRequest};
    use super::*;

    pub async fn req(request_data: RegisterByEmailRequest) -> BackendRes<String> {
        let RegisterByEmailRequest {
            device_id,
            email,
            captcha,
            predecessor_invite_code,
            password,
            encrypted_prikey,
            pubkey,
        } = request_data;
        //captcha::validate_email(&email)?;
        super::register(device_id, email, captcha, predecessor_invite_code, password, ContactType::Email,encrypted_prikey,pubkey).await
    }
}

pub mod by_phone{
    use crate::account_manager::{captcha, RegisterByPhoneRequest};
    use super::*;

    pub async fn req(request_data: RegisterByPhoneRequest) -> BackendRes<String> {
        let RegisterByPhoneRequest {
            device_id,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            encrypted_prikey,
            pubkey,
        } = request_data;
        //captcha::validate_phone(&phone_number)?;
        super::register(device_id,phone_number,captcha,predecessor_invite_code,password,ContactType::PhoneNumber,encrypted_prikey,pubkey).await
    }
}