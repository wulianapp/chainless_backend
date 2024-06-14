use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;

use models::wallet_manage_record::WalletManageRecordEntity;

use crate::utils::{get_user_context, judge_role_by_account, token_auth};
use common::data_structures::{KeyRole, SecretKeyState};
use common::error_code::BackendRes;

use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::{SecretFilter, SecretUpdater};

use blockchain::ContractClient;

use models::secret_store::SecretStoreEntity;
use models::PsqlOp;
use serde::{Deserialize, Serialize};

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
    let (user_id, _, device_id, device_brand) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, mut current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let NewcommerSwitchServantRequest {
        old_servant_pubkey,
        new_servant_pubkey,
        new_servant_prikey_encryped_by_password,
        new_servant_prikey_encryped_by_answer,
        new_device_id,
    } = request_data;

    let newcommer_device =
        DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser(&new_device_id, &user_id))
            .await?
            .into_inner();
    let newcommer_device_role =
        judge_role_by_account(newcommer_device.hold_pubkey.as_deref(), &main_account).await?;
    if newcommer_device_role != KeyRole::Undefined {
        Err(format!(
            "your new_device_id's role  is {},and should be Undefined",
            newcommer_device_role
        ))?;
    }

    //check if stored already
    let origin_secret =
        SecretStoreEntity::find(SecretFilter::ByPubkey(&new_servant_pubkey)).await?;
    if origin_secret.is_empty() {
        let secret_info = SecretStoreEntity::new_with_specified(
            &new_servant_pubkey,
            user_id,
            &new_servant_prikey_encryped_by_password,
            &new_servant_prikey_encryped_by_answer,
        );
        secret_info.insert().await?;
    } else {
        SecretStoreEntity::update_single(
            SecretUpdater::State(SecretKeyState::Incumbent),
            SecretFilter::ByPubkey(&new_servant_pubkey),
        )
        .await?;
    }

    SecretStoreEntity::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&old_servant_pubkey),
    )
    .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeServant(&new_servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&new_device_id, &user_id),
    )
    .await?;
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeUndefined(&old_servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&old_servant_pubkey),
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
    record.insert().await?;
    Ok(None)
}
