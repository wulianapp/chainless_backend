use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::data_structures::KeyRole2;
use common::error_code::{AccountManagerError::*, BackendError};
use common::utils::math::random_num;
use models::airdrop::{AirdropEntity, AirdropFilter};
use models::device_info::DeviceInfoEntity;
//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Usage};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::error_code::BackendRes;
use models::account_manager::{get_next_uid, UserFilter, UserInfoEntity};
use models::general::*;
use models::secret_store::SecretStoreEntity;
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

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
async fn gen_user_id(db_cli: &mut PgLocalCli<'_>) -> Result<u32,BackendError> {
    for _ in 0..10 {
        let num = (random_num() % 9_000_000_000 + 1_000_000_000) as u32;
        if UserInfoEntity::find(UserFilter::ById(&num),db_cli).await?.is_empty(){
            return Ok(num);
        }else {
            warn!("user_id {} already exist",num);
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
    //encrypted_prikey: String,
    //pubkey: String,
) -> BackendRes<String> {
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    //check userinfo form db
    let find_res =
        account_manager::UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact), &mut db_cli)
            .await?;
    if !find_res.is_empty() {
        Err(PhoneOrEmailAlreadyRegister)?;
    }

    //todo: register multi_sig_contract account

    //store user info
    //todo: hash password  again before store
    //pubkey is equal to account id when register
    //fixme:
    //let pubkey = "";
    Captcha::check_user_code(&contact, &captcha, Usage::Register)?;

    let this_user_id = gen_user_id(&mut db_cli).await?;
    let mut view = UserInfoEntity::new_with_specified(this_user_id,&password);
    match contact_type {
        ContactType::PhoneNumber => {
            view.user_info.phone_number = Some(contact.clone());
        }
        ContactType::Email => {
            view.user_info.email = Some(contact.clone());
        }
    }
    view.insert(&mut db_cli).await?;

    //register airdrop
    let predecessor_airdrop = AirdropEntity::find_single(
        AirdropFilter::ByInviteCode(&predecessor_invite_code),
        &mut db_cli,
    )
    .await
    .map_err(|_e| InviteCodeNotExist)?;

    let predecessor_userinfo_id = predecessor_airdrop.airdrop.user_id;
    let predecessor_info =
        UserInfoEntity::find_single(
            UserFilter::ById(&predecessor_userinfo_id), 
            &mut db_cli
        ).await?.into_inner();

    if let Some(main_account) =  predecessor_info.main_account{
        let user_airdrop = AirdropEntity::new_with_specified(
            this_user_id,
            predecessor_info.id,
            &main_account,
        );
        user_airdrop.insert(&mut db_cli).await?;
    }else{
        Err(PredecessorNotSetSecurity)?;
    }


    let device = DeviceInfoEntity::new_with_specified(&device_id, &device_brand, this_user_id);
    device.insert(&mut db_cli).await?;

    db_cli.commit().await?;

    let token = crate::utils::token_auth::create_jwt(this_user_id, &device_id, &device_brand)?;
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
