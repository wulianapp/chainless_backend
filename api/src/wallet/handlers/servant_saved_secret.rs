use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use models::general::get_pg_pool_connect;

use crate::utils::{get_user_context, token_auth};
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
    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, _,device_id,_) = token_auth::validate_credentials(&req,&mut db_cli).await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let role = context.role()?;
    
    super::check_role(role, KeyRole2::Servant)?;

    DeviceInfoEntity::update(
        DeviceInfoUpdater::HolderSaved(true),
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
        &mut db_cli,
    )
    .await?;
    Ok(None::<String>)
}
