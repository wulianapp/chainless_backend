use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use actix_web::HttpRequest;
use blockchain::bridge_on_near::Bridge;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;

use common::data_structures::wallet_namage_record::WalletOperateType;

use common::error_code::{BackendRes, WalletError};

use models::account_manager::{UserFilter, UserUpdater,UserInfoEntity};
use models::airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::SecretStoreEntity;
use models::wallet_manage_record::WalletManageRecordEntity;
use models::PsqlOp;
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
    let (user_id, _, device_id, device_brand) = token_auth::validate_credentials(&req).await?;
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

    Captcha::check_and_delete(&user_id.to_string(), &captcha, Usage::SetSecurity)?;

    //store user info
    let user_info = UserInfoEntity::find_single(UserFilter::ById(&user_id))
        .await?
        .into_inner();

    if user_info.main_account.is_some() {
        Err(WalletError::MainAccountAlreadyExist(
            user_info.main_account.clone().unwrap(),
        ))?;
    }

    let mut multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    //todo:
    let main_account_id = super::gen_random_account_id(&multi_sig_cli).await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    UserInfoEntity::update_single(
        UserUpdater::SecruityInfo(&anwser_indexes, &main_account_id),
        UserFilter::ById(&user_id),
    )
    .await?;

    let master_secret = SecretStoreEntity::new_with_specified(
        &master_pubkey,
        user_info.id,
        &master_prikey_encrypted_by_password,
        &master_prikey_encrypted_by_answer,
    );
    master_secret.insert().await?;

    let sub_account_secret = SecretStoreEntity::new_with_specified(
        &subaccount_pubkey,
        user_info.id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    sub_account_secret.insert().await?;

    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeMaster(&master_pubkey),
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
    )
    .await?;

    let txid = multi_sig_cli
        .init_strategy(
            &master_pubkey,
            &main_account_id,
            &subaccount_pubkey,
            &subaccount_id,
        )
        .await?;
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::CreateAccount,
        &master_pubkey,
        &device_id,
        &device_brand,
        vec![txid],
    );
    record.insert().await?;

    AirdropEntity::update_single(
        AirdropUpdater::AccountId(&main_account_id),
        AirdropFilter::ByUserId(&user_id),
    )
    .await?;

    //注册的时候就把允许跨链的状态设置了
    let mut bridge_cli = ContractClient::<Bridge>::new_update_cli().await?;
    let set_res = bridge_cli.set_user_batch(&main_account_id).await?;
    debug!("set_user_batch txid {} ,{}", set_res, main_account_id);

    info!("new wallet {:#?}  successfully", user_info);
    Ok(None)
}
