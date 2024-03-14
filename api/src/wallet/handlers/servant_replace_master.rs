use actix_web::{web, HttpRequest};
use common::error_code::BackendRes;
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

pub(crate) async fn req(
    req: HttpRequest,
    request_data: CreateMainAccountRequest,
) -> BackendRes<String> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let CreateMainAccountRequest {
        master_pubkey,
        master_prikey_encrypted_by_pwd,
        master_prikey_encrypted_by_answer,
        subaccount_pubkey,
        subaccount_prikey_encryped_by_pwd,
        subaccount_prikey_encryped_by_answer,
        anwser_indexes: sign_pwd_hash,
    } = request_data;

    /***
     * user
     * sign_pwd_hash,secruity_is_seted,main_account
     * secret_store
     * master_prikey && sub_prikey
     * blockchain:
     * init_with_subaccount
     */

    //store user info
    let user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;

    models::general::transaction_begin()?;
    account_manager::UserInfoView::update(
        UserUpdater::SecruityInfo(sign_pwd_hash, true, master_pubkey.clone()),
        UserFilter::ById(user_id),
    )?;

    let master_secret = SecretStoreView::new_with_specified(
        &master_pubkey,
        user_info.id,
        &master_prikey_encrypted_by_pwd,
        &master_prikey_encrypted_by_answer,
    );
    master_secret.insert()?;
    //only main_account need to store device info
    /***
    let device = models::device_info::DeviceInfoView::new_with_specified(
        &device_id,
        &device_brand,
        user_id,
        &master_pubkey,
        true,
    );

    device.insert()?;
    */

    let sub_account_secret = SecretStoreView::new_with_specified(
        &subaccount_pubkey,
        user_info.id,
        &subaccount_prikey_encryped_by_pwd,
        &subaccount_prikey_encryped_by_answer,
    );
    sub_account_secret.insert()?;

    let multi_cli = ContractClient::<MultiSig>::new();

    multi_cli
        .init_strategy(&master_pubkey, &subaccount_pubkey)
        .await?;
    models::general::transaction_commit()?;
    info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
