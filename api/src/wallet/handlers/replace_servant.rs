use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest, ReplaceServantRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::PsqlOp;
use tracing::error;

pub(crate) async fn req(req: HttpRequest, request_data: ReplaceServantRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id,device_id,_) = token_auth::validate_credentials2(&req)?;
    let ReplaceServantRequest {
        main_account,
        old_servant_pubkey,
        new_servant_pubkey,
        new_servant_prikey_encryped_by_pwd,
        new_servant_prikey_encryped_by_answer,
        new_device_id,
    } = request_data;

    models::general::transaction_begin()?;
    //check if stored already
    let origin_secret = SecretStoreView::find(
        SecretFilter::ByPubkey(new_servant_pubkey.clone())
    )?;
    if origin_secret.is_empty(){
        let secret_info = SecretStoreView::new_with_specified(
            &new_servant_pubkey,
            user_id,
            &new_servant_prikey_encryped_by_pwd,
            &new_servant_prikey_encryped_by_answer,
        );
        secret_info.insert()?;
    }else {
        SecretStoreView::update(
            SecretUpdater::State(SecretKeyState::Incumbent), 
            SecretFilter::ByPubkey(new_servant_pubkey.clone())
        )?;
    }

    SecretStoreView::update(
        SecretUpdater::State(SecretKeyState::Abandoned), 
        SecretFilter::ByPubkey(old_servant_pubkey.clone())
    )?;

     //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update(
        DeviceInfoUpdater::BecomeServant(new_servant_pubkey.clone()),
        DeviceInfoFilter::ByDeviceUser(new_device_id,user_id)
    )?;
    DeviceInfoView::update(
        DeviceInfoUpdater::BecomeUndefined(old_servant_pubkey.clone()),
        DeviceInfoFilter::ByHoldKey(old_servant_pubkey.clone())
    )?;

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    //it is impossible to get none
    let mut current_strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.clone()))?;
    //delete older and than add new
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &old_servant_pubkey);

     current_strategy
        .servant_pubkeys
        .push(new_servant_pubkey);

    multi_sig_cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;

    models::general::transaction_commit()?;
    Ok(None::<String>)
}
