use std::collections::HashMap;

use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use actix_web::{web, HttpRequest};
use blockchain::bridge_on_near::Bridge;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::KeyRole2;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use common::error_code::{BackendError, BackendRes, WalletError};
use common::utils::math::generate_random_hex_string;
use models::account_manager::{get_next_uid, UserFilter, UserUpdater};
use models::airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::general::{get_pg_pool_connect, transaction_begin};
use models::secret_store::SecretStoreEntity;
use models::wallet_manage_record::WalletManageRecordEntity;
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing::info;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMainAccountRequest {
    master_pubkey: String,
    master_prikey_encrypted_by_password: String,
    master_prikey_encrypted_by_answer: String,
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_password: String,
    subaccount_prikey_encryped_by_answer: String,
    anwser_indexes: String,
    captcha: String,
}

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

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;
    //store user info
    let user_info =
        account_manager::UserInfoEntity::find_single(UserFilter::ById(user_id), &mut db_cli)
            .await?;

    if !user_info.user_info.main_account.eq("") {
        Err(WalletError::MainAccountAlreadyExist(
            user_info.user_info.main_account.clone(),
        ))?;
    }

    let multi_sig_cli = ContractClient::<MultiSig>::new().await?;
    //todo:
    let main_account_id = super::gen_random_account_id(&multi_sig_cli).await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    account_manager::UserInfoEntity::update_single(
        UserUpdater::SecruityInfo(&anwser_indexes, true, &main_account_id),
        UserFilter::ById(user_id),
        &mut db_cli,
    )
    .await?;

    let master_secret = SecretStoreEntity::new_with_specified(
        &master_pubkey,
        user_info.id,
        &master_prikey_encrypted_by_password,
        &master_prikey_encrypted_by_answer,
    );
    master_secret.insert(&mut db_cli).await?;

    let sub_account_secret = SecretStoreEntity::new_with_specified(
        &subaccount_pubkey,
        user_info.id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    sub_account_secret.insert(&mut db_cli).await?;

    //fixme: 这里遇到过一次没有commit，db事务，但是update_single成功的情况
    debug!("__line_{}", line!());
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeMaster(&master_pubkey),
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
        &mut db_cli,
    )
    .await?;
    debug!("__line_{}", line!());

    let txid = multi_sig_cli
        .init_strategy(
            &master_pubkey,
            &main_account_id,
            &subaccount_pubkey,
            &subaccount_id,
        )
        .await?;

    debug!("__line_{}", line!());
    let record = WalletManageRecordEntity::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::CreateAccount,
        &master_pubkey,
        &device_id,
        &device_brand,
        vec![txid],
    );
    record.insert(&mut db_cli).await?;

    AirdropEntity::update_single(
        AirdropUpdater::AccountId(&main_account_id),
        AirdropFilter::ByUserId(&user_id.to_string()),
        &mut db_cli,
    )
    .await?;

    //注册的时候就把允许跨链的状态设置了
    let bridge_cli = ContractClient::<Bridge>::new().await?;
    let set_res = bridge_cli.set_user_batch(&main_account_id).await?;
    debug!("set_user_batch txid {} ,{}", set_res, main_account_id);

    db_cli.commit().await?;
    info!("new wallet {:#?}  successfully", user_info);
    Ok(None)
}
