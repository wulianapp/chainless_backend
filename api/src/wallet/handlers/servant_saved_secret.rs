use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use models::general::get_pg_pool_connect;

use crate::utils::token_auth;
use common::data_structures::KeyRole2;
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::secret_store::{SecretFilter, SecretUpdater};

use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreEntity;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;
    let (_user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Servant)?;

    DeviceInfoEntity::update(
        DeviceInfoUpdater::HolderSaved(true),
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?;
    Ok(None::<String>)
}
