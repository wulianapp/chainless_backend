use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::data_structures::KeyRole2;
use common::error_code::AccountManagerError::*;
use models::device_info::DeviceInfoView;
//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Usage};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::error_code::BackendRes;
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};
use models::secret_store::SecretStoreView;
use models::{account_manager, secret_store, PsqlOp};
use tracing::{debug, info};

async fn register(
    device_id: String,
    device_brand: String,
    contact: String,
    captcha: String,
    predecessor_invite_code: Option<String>,
    password: String,
    contact_type: ContactType,
    //encrypted_prikey: String,
    //pubkey: String,
) -> BackendRes<String> {
    //check userinfo form db
    let find_res = account_manager::UserInfoView::find(UserFilter::ByPhoneOrEmail(&contact))?;
    if !find_res.is_empty() {
        Err(PhoneOrEmailAlreadyRegister)?;
    }

    //todo: register multi_sig_contract account

    //store user info
    let this_user_id = get_next_uid()?;
    debug!("this_user_id _______{}", this_user_id);
    //todo: hash password  again before store
    //pubkey is equal to account id when register
    //fixme:
    //let pubkey = "";
    let mut view = UserInfoView::new_with_specified(&password, &this_user_id.to_string());
    match contact_type {
        ContactType::PhoneNumber => {
            view.user_info.phone_number = contact.clone();
        }
        ContactType::Email => {
            view.user_info.email = contact.clone();
        }
    }

    if let Some(code) = predecessor_invite_code {
        let predecessor = UserInfoView::find_single(UserFilter::ByInviteCode(&code))
            .map_err(|_e| InviteCodeNotExist)?;
        if !predecessor.user_info.secruity_is_seted {
            Err(PredecessorNotSetSecurity)?;
        }
        view.user_info.predecessor = Some(predecessor.id);
    }

    Captcha::check_user_code(&contact, &captcha, Usage::Register)?;
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
    //let device = models::device_info::DeviceInfoView::new_with_specified(&device_id, &device_brand,this_user_id, &pubkey,true);
    //device.insert()?;
    let device = DeviceInfoView::new_with_specified(&device_id, &device_brand, this_user_id);
    device.insert()?;

    models::general::transaction_commit()?;

    let token = crate::utils::token_auth::create_jwt(this_user_id, &device_id, &device_brand);
    info!("user {:?} register successfully", view.user_info);
    Ok(Some(token))
}

pub mod by_email {
    use super::*;
    use crate::account_manager::RegisterByEmailRequest;

    pub async fn req(request_data: RegisterByEmailRequest) -> BackendRes<String> {
        let RegisterByEmailRequest {
            device_id,
            device_brand,
            email,
            captcha,
            predecessor_invite_code,
            password,
            //encrypted_prikey,
            //pubkey,
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
            //encrypted_prikey,
            //pubkey,
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
            device_brand,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            //encrypted_prikey,
            //pubkey,
        } = request_data;
        //captcha::validate_phone(&phone_number)?;
        super::register(
            device_id,
            device_brand,
            phone_number,
            captcha,
            predecessor_invite_code,
            password,
            ContactType::PhoneNumber,
            //encrypted_prikey,
            //pubkey,
        )
        .await
    }
}
