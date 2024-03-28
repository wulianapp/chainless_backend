use actix_web::{web, HttpRequest};
use common::data_structures::KeyRole2;
use common::error_code::{BackendError, BackendRes};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, ReconfirmSendMoneyRequest};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use models::account_manager::{get_next_uid, UserFilter, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use tracing::info;
use common::error_code::BackendError::ChainError;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: CreateMainAccountRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let CreateMainAccountRequest {
        master_pubkey,
        master_prikey_encrypted_by_password,
        master_prikey_encrypted_by_answer,
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        anwser_indexes,
        captcha
    } = request_data;

    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::SetSecurity)?;


    //store user info
    let user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    
    if user_info.user_info.main_account != ""{
        Err(BackendError::InternalError("main_account is already existent".to_string()))?;
    }


    models::general::transaction_begin()?;
    account_manager::UserInfoView::update(
        UserUpdater::SecruityInfo(&anwser_indexes, true, &master_pubkey),
        UserFilter::ById(user_id),
    )?;

    let master_secret = SecretStoreView::new_with_specified(
        &master_pubkey,
        user_info.id,
        &master_prikey_encrypted_by_password,
        &master_prikey_encrypted_by_answer,
    );
    master_secret.insert()?;

    let sub_account_secret = SecretStoreView::new_with_specified(
        &subaccount_pubkey,
        user_info.id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    sub_account_secret.insert()?;

    DeviceInfoView::update(
        DeviceInfoUpdater::BecomeMaster(&master_pubkey),
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
    )?;

    let multi_cli = ContractClient::<MultiSig>::new()?;

    multi_cli
        .init_strategy(&master_pubkey, &subaccount_pubkey)
        .await?;
    models::general::transaction_commit()?;
    info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
