use actix_web::{HttpRequest, web};
use common::error_code::BackendRes;
use models::secret_store::SecretStoreView;
//use log::info;
use tracing::info;
use blockchain::ContractClient;
use blockchain::multi_sig::MultiSig;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister};
use models::{account_manager, PsqlOp, secret_store};
use models::account_manager::{get_next_uid, UserFilter, UserUpdater};
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, ReconfirmSendMoneyRequest};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: CreateMainAccountRequest,
) -> BackendRes<String> {

    let user_id = token_auth::validate_credentials(&req)?;
    let CreateMainAccountRequest{
        master_pubkey,
        master_prikey_encrypted_by_pwd,
        master_prikey_by_answer,
        subaccount_pubkey,
        subaccount_prikey_encryped_by_answer,
        subaccount_prikey_encryped_by_pwd,
        sign_pwd_hash,
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
    let mut user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;

    models::general::transaction_begin()?;
    account_manager::UserInfoView::update(
        UserUpdater::SecruityInfo(sign_pwd_hash,true,master_pubkey.clone()),
    UserFilter::ById(user_id))?;

    let master_secret = SecretStoreView::new_with_specified(
        &master_pubkey,
        user_info.id,
        &master_prikey_encrypted_by_pwd,
        &master_prikey_by_answer
    );
    master_secret.insert()?;
    let sub_account_secret = SecretStoreView::new_with_specified(
        &subaccount_pubkey,
        user_info.id,
        &subaccount_prikey_encryped_by_pwd,
        &subaccount_prikey_encryped_by_answer
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