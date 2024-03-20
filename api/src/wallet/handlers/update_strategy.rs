use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::data_structures::KeyRole2;
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    PsqlOp,
};

use crate::utils::token_auth;
use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest, request_data: web::Json<UpdateStrategy>) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let UpdateStrategy {
        strategy,
    } = request_data.0;
    let main_account = super::get_main_account(user_id)?;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

    //fixme:
    let strategy = strategy
        .into_iter()
        .map(|x| MultiSigRank {
            min: x.min,
            max_eq: x.max_eq,
            sig_num: x.sig_num,
        })
        .collect();

    //add wallet info
    let multi_sig_cli = ContractClient::<MultiSig>::new();
    multi_sig_cli.update_rank(&main_account, strategy).await?;

    Ok(None::<String>)
}
