use std::collections::BTreeMap;

use actix_web::{web, HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole2, SecretKeyType};
use common::utils::math::coin_amount::display2raw;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::{get_db_pool_connect, transaction_begin, transaction_commit};
use models::wallet_manage_record::WalletManageRecordView;
//use log::info;
use crate::utils::token_auth;
use crate::wallet::{AddSubaccountRequest, CreateMainAccountRequest, ReconfirmSendMoneyRequest};
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::secret_store::SecretStoreView;
use models::{account_manager, secret_store, PsqlOp};
use tracing::info;

pub async fn req(req: HttpRequest, request_data: AddSubaccountRequest) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let AddSubaccountRequest {
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        hold_value_limit,
    } = request_data;
    super::have_no_uncompleted_tx(&main_account)?;
    let hold_value_limit = display2raw(&hold_value_limit)?;
    let (_, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    //todo: 24小时内只能三次增加的限制

    //store user info
    //let mut user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;
    //user_info.user_info.account_ids.push(pubkey.clone());

    let mut conn = get_db_pool_connect()?;
    let mut trans = transaction_begin(&mut conn)?;

    //account_manager::UserInfoView::update_single(UserUpdater::AccountIds(user_info.user_info.account_ids.clone()),UserFilter::ById(user_id))?;
    let multi_sig_cli = ContractClient::<MultiSig>::new().await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    //todo: encrypted_prikey_by_password
    let secret = SecretStoreView::new_with_specified(
        &subaccount_pubkey,
        user_id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    secret.insert_with_trans(&mut trans)?;

    let multi_cli = ContractClient::<MultiSig>::new().await?;
    let sub_confs = BTreeMap::from([(
        subaccount_id.as_str(),
        SubAccConf {
            pubkey: subaccount_pubkey,
            hold_value_limit,
        },
    )]);
    let txid = multi_cli.add_subaccount(&main_account, sub_confs).await?;

    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::AddSubaccount,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert_with_trans(&mut trans)?;

    //multi_cli.add_subaccount(user_info.user_info., subacc)1
    transaction_commit(trans)?;
    //info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
