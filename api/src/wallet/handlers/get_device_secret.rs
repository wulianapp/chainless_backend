use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    secret_store::{SecretFilter, SecretStoreView},
    PsqlOp,
};

use crate::utils::token_auth;
use common::{
    data_structures::secret_store::SecretStore,
    error_code::{BackendError, BackendRes},
};
use serde::{Deserialize, Serialize};

use crate::wallet::GetStrategyRequest;

/***
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: Vec<String>,
}

pub(crate) async fn req(
    req: HttpRequest
) -> BackendRes<SecretStore> {
    let (user_id,device_id,_) = token_auth::validate_credentials2(&req)?;

    let pubkey  = DeviceInfoView::find_single(
        DeviceInfoFilter::ByDeviceUser(device_id, user_id)
    )?
    .device_info
    .hold_pubkey
    .ok_or(BackendError::InternalError("this haven't be servant yet".to_string()))?;

    let secret =
     SecretStoreView::find_single(SecretFilter::ByPubkey(pubkey))?;
    Ok(Some(secret.secret_store))
}
*/
