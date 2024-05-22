use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;
use models::general::{get_db_pool_connect, transaction_begin, transaction_commit};
use models::wallet_manage_record::WalletManageRecordView;

use crate::account_manager::user_info;
use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

use super::get_role;

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let AddServantRequest {
        servant_pubkey,
        servant_prikey_encryped_by_password,
        servant_prikey_encryped_by_answer,
        holder_device_id,
        holder_device_brand: _,
    } = request_data;
    let (user, mut current_strategy, device) =
        super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    if current_strategy.servant_pubkeys.len() >= 11 {
        Err(WalletError::ServantNumReachLimit)?;
    }

    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let mut conn = get_db_pool_connect()?;
    let mut trans =  models::general::transaction_begin(&mut conn)?;

    //如果之前就有了，说明之前曾经被赋予过master或者servant的身份
    let origin_secret = SecretStoreView::find(SecretFilter::ByPubkey(&servant_pubkey))?;
    if origin_secret.is_empty() {
        let secret_info = SecretStoreView::new_with_specified(
            &servant_pubkey,
            user_id,
            &servant_prikey_encryped_by_password,
            &servant_prikey_encryped_by_answer,
        );
        secret_info.insert_with_trans(&mut trans)?;
    } else {
        SecretStoreView::update_single_with_trans(
            SecretUpdater::State(SecretKeyState::Incumbent),
            SecretFilter::ByPubkey(&servant_pubkey),
            &mut trans
        )?;
    }

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new().await?;
    //it is impossible to get none

    current_strategy
        .servant_pubkeys
        .push(servant_pubkey.clone());
    let txid = multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update_single_with_trans(
        DeviceInfoUpdater::AddServant(&servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&holder_device_id, user_id),
        &mut trans
    )?;

    //WalletManageRecordView
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::AddServant,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert_with_trans(&mut trans)?;

    transaction_commit(trans)?;
    Ok(None::<String>)
}
