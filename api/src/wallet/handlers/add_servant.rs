use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;
use models::wallet_manage_record::WalletManageRecordEntity;

use crate::utils::{get_user_context, token_auth};
use common::data_structures::{KeyRole, SecretKeyState};
use common::error_code::BackendRes;
use common::error_code::WalletError;

use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::{SecretFilter, SecretUpdater};

use blockchain::ContractClient;

use models::secret_store::SecretStoreEntity;
use models::PsqlOp;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddServantRequest {
    servant_pubkey: String,
    servant_prikey_encryped_by_password: String,
    servant_prikey_encryped_by_answer: String,
    holder_device_id: String,
    holder_device_brand: String,
}

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let AddServantRequest {
        servant_pubkey,
        servant_prikey_encryped_by_password,
        servant_prikey_encryped_by_answer,
        holder_device_id,
        holder_device_brand: _,
    } = request_data;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, mut current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;

    super::have_no_uncompleted_tx(&main_account).await?;

    if current_strategy.servant_pubkeys.len() >= 11 {
        Err(WalletError::ServantNumReachLimit)?;
    }

    //如果之前就有了，说明之前曾经被赋予过master或者servant的身份
    let origin_secret = SecretStoreEntity::find(SecretFilter::ByPubkey(&servant_pubkey)).await?;
    if origin_secret.is_empty() {
        let secret_info = SecretStoreEntity::new_with_specified(
            &servant_pubkey,
            user_id,
            &servant_prikey_encryped_by_password,
            &servant_prikey_encryped_by_answer,
        );
        secret_info.insert().await?;
    } else {
        SecretStoreEntity::update_single(
            SecretUpdater::State(SecretKeyState::Incumbent),
            SecretFilter::ByPubkey(&servant_pubkey),
        )
        .await?;
    }

    //add wallet info
    let mut multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    //it is impossible to get none

    current_strategy
        .servant_pubkeys
        .push(servant_pubkey.clone());
    let txid = multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::AddServant(&servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&holder_device_id, &user_id),
    )
    .await?;

    //WalletManageRecordView
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::AddServant,
        &context.device.hold_pubkey.unwrap(),
        &context.device.id,
        &context.device.brand,
        vec![txid],
    );
    record.insert().await?;

    Ok(None)
}
