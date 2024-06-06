use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;
use models::general::get_pg_pool_connect;
use models::wallet_manage_record::WalletManageRecordEntity;

use crate::utils::{get_user_context, judge_role_by_account, token_auth};
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::{SecretFilter, SecretUpdater};

use blockchain::ContractClient;
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreEntity;
use models::{PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewcommerSwitchServantRequest {
    old_servant_pubkey: String,
    new_servant_pubkey: String,
    new_servant_prikey_encryped_by_password: String,
    new_servant_prikey_encryped_by_answer: String,
    new_device_id: String,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: NewcommerSwitchServantRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, device_brand) = token_auth::validate_credentials(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;


    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (main_account,mut current_strategy) = context.account_strategy()?;
    let role = context.role()?;
    
    super::check_role(role, KeyRole2::Master)?;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;

    let NewcommerSwitchServantRequest {
        old_servant_pubkey,
        new_servant_pubkey,
        new_servant_prikey_encryped_by_password,
        new_servant_prikey_encryped_by_answer,
        new_device_id,
    } = request_data;

    let newcommer_device = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&new_device_id, &user_id),
        &mut db_cli,
    )
    .await?.into_inner();
    let newcommer_device_role = judge_role_by_account(newcommer_device.hold_pubkey.as_deref(),&main_account).await?;
    if newcommer_device_role != KeyRole2::Undefined {
        Err(format!(
            "your new_device_id's role  is {},and should be Undefined",
            newcommer_device_role
        ))?;
    }

    //check if stored already
    let origin_secret =
        SecretStoreEntity::find(SecretFilter::ByPubkey(&new_servant_pubkey), &mut db_cli).await?;
    if origin_secret.is_empty() {
        let secret_info = SecretStoreEntity::new_with_specified(
            &new_servant_pubkey,
            user_id,
            &new_servant_prikey_encryped_by_password,
            &new_servant_prikey_encryped_by_answer,
        );
        secret_info.insert(&mut db_cli).await?;
    } else {
        SecretStoreEntity::update_single(
            SecretUpdater::State(SecretKeyState::Incumbent),
            SecretFilter::ByPubkey(&new_servant_pubkey),
            &mut db_cli,
        )
        .await?;
    }

    SecretStoreEntity::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&old_servant_pubkey),
        &mut db_cli,
    )
    .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeServant(&new_servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&new_device_id, &user_id),
        &mut db_cli,
    )
    .await?;
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeUndefined(&old_servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&old_servant_pubkey),
        &mut db_cli,
    )
    .await?;

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;

    //delete older and than add new
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &old_servant_pubkey);

    current_strategy.servant_pubkeys.push(new_servant_pubkey);

    let tx_id = multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::NewcomerSwitchServant,
        &old_servant_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert(&mut db_cli).await?;
    db_cli.commit().await?;
    Ok(None::<String>)
}
