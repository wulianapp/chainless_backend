use std::collections::HashMap;

use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::KeyRole2;
use common::error_code::{BackendError, BackendRes};
use common::utils::math::generate_random_hex_string;
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::SecretStoreView;
use models::wallet_manage_record::WalletManageRecordView;
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
use common::error_code::BackendError::ChainError;
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
        master_prikey_encrypted_by_password,
        master_prikey_encrypted_by_answer,
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        anwser_indexes,
        captcha,
    } = request_data;

    Captcha::check_user_code(&user_id.to_string(), &captcha, Usage::SetSecurity)?;

    //store user info
    let user_info = account_manager::UserInfoView::find_single(UserFilter::ById(user_id))?;

    if !user_info.user_info.main_account.eq("") {
        Err(BackendError::InternalError(
            "main_account is already existent".to_string(),
        ))?;
    }

    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    //todo:
    let main_account_id = super::gen_random_account_id(&multi_sig_cli).await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    models::general::transaction_begin()?;
    account_manager::UserInfoView::update_single(
        UserUpdater::SecruityInfo(&anwser_indexes, true, &main_account_id),
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

    DeviceInfoView::update_single(
        DeviceInfoUpdater::BecomeMaster(&master_pubkey),
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
    )?;

    let txid = multi_sig_cli
        .init_strategy(
            &master_pubkey,
            &main_account_id,
            &subaccount_pubkey,
            &subaccount_id,
        )
        .await?;
    //todo: 通过get_user进行检查、在里面了就不调用了
    let bridge_cli = ContractClient::<Bridge>::new().unwrap();
    let set_res = bridge_cli.set_user_batch(&main_account_id).await;
    println!(
        "set_user_batch txid {} ,{}",
        set_res.unwrap(),
        main_account_id
    );

    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::CreateAccount,
        &master_pubkey,
        &device_id,
        &device_brand,
        vec![txid],
    );
    record.insert()?;

    //todo: 跨链桥更新状态

    models::general::transaction_commit()?;
    info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
