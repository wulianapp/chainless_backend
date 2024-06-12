use std::collections::BTreeMap;

use actix_web::{web, HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::KeyRole;
use common::utils::math::coin_amount::display2raw;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::{get_pg_pool_connect, transaction_begin, transaction_commit};
use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;
use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};
use models::secret_store::SecretStoreEntity;
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSubaccountRequest {
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_password: String,
    subaccount_prikey_encryped_by_answer: String,
    hold_value_limit: String,
}

pub async fn req(req: HttpRequest, request_data: AddSubaccountRequest) -> BackendRes<String> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let AddSubaccountRequest {
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        hold_value_limit,
    } = request_data;
    let hold_value_limit =
        display2raw(&hold_value_limit).map_err(|_e| WalletError::UnSupportedPrecision)?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    //todo: 24小时内只能三次增加的限制

    //account_manager::UserInfoView::update_single(UserUpdater::AccountIds(user_info.user_info.account_ids.clone()),UserFilter::ById(&user_id))?;
    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    //todo: encrypted_prikey_by_password
    let secret = SecretStoreEntity::new_with_specified(
        &subaccount_pubkey,
        user_id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    secret.insert().await?;

    let multi_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let sub_confs = BTreeMap::from([(
        subaccount_id.as_str(),
        SubAccConf {
            pubkey: subaccount_pubkey,
            hold_value_limit,
        },
    )]);
    let txid = multi_cli.add_subaccount(&main_account, sub_confs).await?;

    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::AddSubaccount,
        &context.device.hold_pubkey.unwrap(),
        &context.device.id,
        &context.device.brand,
        vec![txid],
    );
    record.insert().await?;

    //multi_cli.add_subaccount(user_info.user_info., subacc)1
    //info!("new wallet {:?}  successfully", user_info);
    Ok(None::<String>)
}
