use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::KeyRole2;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, GenTxNewcomerReplaceMasterRequest, ReconfirmSendMoneyRequest};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use tracing::{error, info};
use serde::{Deserialize,Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenReplaceKeyInfo {
    pub add_key_txid: String,
    pub add_key_raw: String,
    pub delete_key_txid: String,
    pub delete_key_raw: String,
}
pub(crate) async fn req(
    req: HttpRequest
) -> BackendRes<GenReplaceKeyInfo> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    
    let servant_pubkey  = DeviceInfoView::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id)
    )?
    .device_info
    .hold_pubkey
    .ok_or(BackendError::InternalError("this haven't be servant yet".to_string()))?;


    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Servant{
        Err(WalletError::UneligiableRole(device.device_info.key_role, KeyRole2::Servant))?;
    }


    let client = ContractClient::<MultiSig>::new();
    let master_pubkey = client.get_master_pubkey(&main_account).await;
    
    let (add_key_txid,add_key_raw) = client.add_key(&main_account, &servant_pubkey).await.unwrap().unwrap();
    let (delete_key_txid,delete_key_raw) = client.delete_key(&main_account, &master_pubkey).await.unwrap().unwrap();
    let replace_txids = GenReplaceKeyInfo{
        add_key_txid,
        add_key_raw,
        delete_key_txid,
        delete_key_raw
    };
    Ok(Some(replace_txids))
  
}
