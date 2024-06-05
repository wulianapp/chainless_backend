use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;
use models::general::{get_pg_pool_connect, transaction_begin};
use models::wallet_manage_record::WalletManageRecordEntity;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::{SecretFilter, SecretUpdater};

use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreEntity;
use models::{PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveServantRequest {
    servant_pubkey: String,
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: RemoveServantRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, device_brand) = token_auth::validate_credentials(&req)?;
    let RemoveServantRequest { servant_pubkey } = request_data;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    let (user, mut current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account.clone().unwrap();
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    //old key_store set abandoned
    SecretStoreEntity::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&servant_pubkey),
        &mut db_cli,
    )
    .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoEntity::update_single(
        DeviceInfoUpdater::BecomeUndefined(&servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&servant_pubkey),
        &mut db_cli,
    )
    .await?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new_update_cli().await?;
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &servant_pubkey);
    let tx_id = cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::RemoveServant,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert(&mut db_cli).await?;
    db_cli.commit().await?;
    Ok(None::<String>)
}
