use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest, RemoveServantRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: RemoveServantRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let RemoveServantRequest { servant_pubkey } = request_data;
    let (user, mut current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    models::general::transaction_begin()?;

    //old key_store set abandoned
    SecretStoreView::update(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&servant_pubkey),
    )?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new()?;
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &servant_pubkey);
    cli.update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update(
        DeviceInfoUpdater::BecomeUndefined(&servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&servant_pubkey),
    )?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
