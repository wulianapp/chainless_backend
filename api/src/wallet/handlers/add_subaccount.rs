use std::collections::HashMap;

use actix_web::{web, HttpRequest};
use common::data_structures::{KeyRole2, SecretKeyType};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
//use log::info;
use crate::utils::token_auth;
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::secret_store::SecretStoreView;
use models::{account_manager, secret_store, PsqlOp};
use tracing::info;
use crate::wallet::{AddSubaccountRequest, CreateMainAccountRequest, ReconfirmSendMoneyRequest};

pub async fn req(req: HttpRequest, request_data: AddSubaccountRequest) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;
    let AddSubaccountRequest {
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        hold_value_limit,
    } = request_data;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

    //todo: 24小时内只能三次增加的限制    

    //store user info
    //let mut user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    //user_info.user_info.account_ids.push(pubkey.clone());

    models::general::transaction_begin()?;
    //account_manager::UserInfoView::update(UserUpdater::AccountIds(user_info.user_info.account_ids.clone()),UserFilter::ById(user_id))?;

    //todo: encrypted_prikey_by_password
    let secret = SecretStoreView::new_with_specified(
        &subaccount_pubkey,
        user_id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    secret.insert()?;
    let multi_cli = ContractClient::<MultiSig>::new();
    let sub_confs = HashMap::from([(subaccount_pubkey.as_str(),SubAccConf{ hold_value_limit})]);
    multi_cli
        .add_subaccount(&main_account, sub_confs)
        .await
        .unwrap();
    //multi_cli.add_subaccount(user_info.user_info., subacc)1
    models::general::transaction_commit()?;
    //info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
