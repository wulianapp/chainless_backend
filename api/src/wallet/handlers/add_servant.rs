use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::AddServantRequest;
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(req: HttpRequest, request_data: AddServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id,device_id,_) = token_auth::validate_credentials2(&req)?;
    let AddServantRequest {
        main_account,
        servant_pubkey,
        servant_prikey_encryped_by_password,
        servant_prikey_encryped_by_answer,
        holder_device_id,
        holder_device_brand: _,
    } = request_data;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master{
        Err(WalletError::UneligiableRole(device.device_info.key_role, KeyRole2::Master))?;
    }


    models::general::transaction_begin()?;
    //如果之前就有了，说明之前曾经被赋予过master或者servant的身份
    let origin_secret = SecretStoreView::find(
        SecretFilter::ByPubkey(&servant_pubkey)
    )?;
    if origin_secret.is_empty(){
        let secret_info = SecretStoreView::new_with_specified(
            &servant_pubkey,
            user_id,
            &servant_prikey_encryped_by_password,
            &servant_prikey_encryped_by_answer,
        );
        secret_info.insert()?;
    }else {
        SecretStoreView::update(
            SecretUpdater::State(SecretKeyState::Incumbent), 
            SecretFilter::ByPubkey(&servant_pubkey)
        )?;
    }

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.clone()))?;
    current_strategy
        .servant_pubkeys
        .push(servant_pubkey.clone());
    multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update(
        DeviceInfoUpdater::AddServant(&servant_pubkey),
        DeviceInfoFilter::ByDeviceUser(&holder_device_id,user_id)
    )?;
    



    models::general::transaction_commit()?;
    Ok(None::<String>)
}
