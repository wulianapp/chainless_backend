use actix_web::{HttpRequest, web};
//use log::info;
use tracing::info;
use blockchain::ContractClient;
use blockchain::multi_sig::MultiSig;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister};
use common::http::{BackendRes, token_auth};
use models::{account_manager, PsqlOp, secret_store};
use models::account_manager::{get_next_uid, UserFilter, UserUpdater};
use models::secret_store::SecretStore2;
use crate::account_manager::captcha::{Captcha, ContactType, Usage};
use crate::wallet::{NewMasterRequest, ReconfirmSendMoneyRequest};

pub async fn req(
    req: HttpRequest,
    request_data: NewMasterRequest,
) -> BackendRes<String> {

    let user_id = token_auth::validate_credentials(&req)?;
    let NewMasterRequest{encrypted_prikey,pubkey} = request_data;
    //todo: check if is master

    //store user info
    let mut user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    user_info.user_info.account_ids.push(pubkey.clone());


    models::general::transaction_begin()?;
    account_manager::UserInfoView::update(UserUpdater::AccountIds(user_info.user_info.account_ids.clone()),UserFilter::ById(user_id))?;

    let secret = SecretStore2::new_with_specified(pubkey.clone(), user_id, encrypted_prikey);
    secret.insert()?;
    let multi_cli = ContractClient::<MultiSig>::new();

    multi_cli
        .init_strategy(&pubkey)
        .await
        .unwrap();
    //multi_cli.add_subaccount(user_info.user_info., subacc)1
    models::general::transaction_commit()?;
    info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}