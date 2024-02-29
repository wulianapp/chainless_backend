use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::*;
//use log::{debug, info};
use tracing::{debug,info};
use crate::account_manager::captcha::{Captcha, ContactType, Usage};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::http::BackendRes;
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};
use models::{account_manager, PsqlOp, secret_store};
use models::secret_store::SecretStoreView;

async fn register(
    device_id: String,
    contact: String,
    captcha: String,
    predecessor_invite_code: Option<String>,
    password: String,
    contact_type: ContactType,
    //encrypted_prikey: String,
    pubkey: String,
) -> BackendRes<String> {
    Captcha::check_user_code(&contact, &captcha, Usage::Register)?;

    //check userinfo form db
    let find_res = account_manager::UserInfoView::find(UserFilter::ByPhoneOrEmail(contact.clone()))?;
    if !find_res.is_empty(){
        Err(PhoneOrEmailAlreadyRegister)?;
    }


    //todo: register multi_sig_contract account

    //store user info
    let this_user_id = get_next_uid()?;
    debug!("this_user_id _______{}", this_user_id);
    //todo: hash password  again before store
    //pubkey is equal to account id when register
    let mut view = UserInfoView::new_with_specified(&password,&this_user_id.to_string(),&pubkey);
    match contact_type {
        ContactType::PhoneNumber => {
            view.user_info.phone_number = contact;
        }
        ContactType::Email => {
            view.user_info.email = contact;
        }
    }

    if let Some(code) = predecessor_invite_code {
        let predecessor = UserInfoView::find_single(UserFilter::ByInviteCode(code)).map_err(|e|InviteCodeNotExist)?;
        view.user_info.predecessor = Some(predecessor.id);
    }

    models::general::transaction_begin()?;
    //account_manager::single_insert(&view.user_info)?;
    account_manager::UserInfoView::insert(&view)?;
    /***
    let secret = SecretStore2::new_with_specified(pubkey.clone(), this_user_id, encrypted_prikey);
    secret.insert()?;
    ***/
    //注册多签账户放在安全问答之后
    //let multi_cli = ContractClient::<MultiSig>::new();
    //multi_cli.init_strategy(&pubkey).await.unwrap();
    models::general::transaction_commit()?;

    let token = common::http::token_auth::create_jwt(this_user_id, device_id);
    info!("user {:?} register successfully", view.user_info);
    Ok(Some(token))
}

pub mod by_email {
    use super::*;
    use crate::account_manager::RegisterByEmailRequest;

    pub async fn req(request_data: RegisterByEmailRequest) -> BackendRes<String> {
        let RegisterByEmailRequest {
            device_id,
            email,
            captcha,
            predecessor_invite_code,
            password,
            //encrypted_prikey,
            pubkey,
        } = request_data;
        //captcha::validate_email(&email)?;
        super::register(
            device_id,
            email,
            captcha,
            predecessor_invite_code,
            password,
            ContactType::Email,
            //encrypted_prikey,
            pubkey,
        )
        .await
    }
}

pub mod by_phone {
    use super::*;
    use crate::account_manager::RegisterByPhoneRequest;

    pub async fn req(request_data: RegisterByPhoneRequest) -> BackendRes<String> {
        let RegisterByPhoneRequest {
            device_id,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            //encrypted_prikey,
            pubkey,
        } = request_data;
        //captcha::validate_phone(&phone_number)?;
        super::register(
            device_id,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            ContactType::PhoneNumber,
            //encrypted_prikey,
            pubkey,
        )
        .await
    }
}
