use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::SecretKeyType;
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest, ServantSavedSecretRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ServantSavedSecretRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    //理论上设备和userid可以锁定唯一的pubkey，不需要再传，
    //但是也要考虑比如从设备成为主设备这种发生pubkey更换的场景，这个场景需要把老pubkey进行删除
    let _servant_pubkey = request_data.servant_pubkey;
    DeviceInfoView::update(
        DeviceInfoUpdater::HolderSaved(true),
        DeviceInfoFilter::ByDeviceUser(device_id, user_id),
    )?;
    Ok(None::<String>)
}
