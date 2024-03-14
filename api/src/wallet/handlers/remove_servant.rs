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

pub(crate) async fn req(req: HttpRequest, request_data: RemoveServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id,device_id,_) = token_auth::validate_credentials2(&req)?;
    let RemoveServantRequest {
        servant_pubkey,
    } = request_data;

    let user = UserInfoView::find_single(UserFilter::ById(user_id))?;

    models::general::transaction_begin()?;

    //old key_store set abandoned
    SecretStoreView::update(
        SecretUpdater::State(SecretKeyState::Abandoned), 
        SecretFilter::ByPubkey(&servant_pubkey)
    )?;



    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli
        .get_strategy(&user.user_info.main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(user.user_info.main_account.clone()))?;
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &servant_pubkey);
    multi_sig_cli
        .update_servant_pubkey(&user.user_info.main_account, current_strategy.servant_pubkeys)
        .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update(
        DeviceInfoUpdater::BecomeUndefined(&servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&device_id,user_id)
    )?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
