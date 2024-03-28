use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(
    req: HttpRequest
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Servant)?;

    DeviceInfoView::update(
        DeviceInfoUpdater::HolderSaved(true),
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
    )?;
    Ok(None::<String>)
}
