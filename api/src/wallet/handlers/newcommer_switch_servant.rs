use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest, NewcommerSwitchServantRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;
use common::error_code::BackendError::ChainError;


pub(crate) async fn req(
    req: HttpRequest,
    request_data: NewcommerSwitchServantRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
  
    let (user,current_strategy,device) = 
        super::get_session_state(user_id,&device_id).await?;
        let main_account = user.main_account;
        super::have_no_uncompleted_tx(&main_account)?;
        let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
        super::check_role(current_role,KeyRole2::Master)?;

      

    let NewcommerSwitchServantRequest {
        old_servant_pubkey,
        new_servant_pubkey,
        new_servant_prikey_encryped_by_password,
        new_servant_prikey_encryped_by_answer,
        new_device_id,
    } = request_data;

    let undefined_device = DeviceInfoView::find_single(
        DeviceInfoFilter::ByDeviceUser(&new_device_id, user_id
        ))?;
    if undefined_device.device_info.key_role !=  KeyRole2::Undefined{
        Err(BackendError::InternalError(
            format!("your new_device_id's role  is {},and should be Undefined",
            undefined_device.device_info.key_role)
        ))?;
    }   

 

    models::general::transaction_begin()?;
    //check if stored already
    let origin_secret = SecretStoreView::find(SecretFilter::ByPubkey(&new_servant_pubkey))?;
    if origin_secret.is_empty() {
        let secret_info = SecretStoreView::new_with_specified(
            &new_servant_pubkey,
            user_id,
            &new_servant_prikey_encryped_by_password,
            &new_servant_prikey_encryped_by_answer,
        );
        secret_info.insert()?;
    } else {
        SecretStoreView::update_single(
            SecretUpdater::State(SecretKeyState::Incumbent),
            SecretFilter::ByPubkey(&new_servant_pubkey),
        )?;
    }

    SecretStoreView::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&old_servant_pubkey),
    )?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update_single(
        DeviceInfoUpdater::BecomeServant(&new_servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&new_device_id, user_id),
    )?;
    DeviceInfoView::update_single(
        DeviceInfoUpdater::BecomeUndefined(&old_servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&old_servant_pubkey),
    )?;

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new().map_err(|err| ChainError(err.to_string()))?;
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await.map_err(|err| ChainError(err.to_string()))?
        .ok_or(WalletError::MainAccountNotExist(main_account.clone()))?;
    //delete older and than add new
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &old_servant_pubkey);

    current_strategy.servant_pubkeys.push(new_servant_pubkey);

    multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await.map_err(|err| ChainError(err.to_string()))?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
