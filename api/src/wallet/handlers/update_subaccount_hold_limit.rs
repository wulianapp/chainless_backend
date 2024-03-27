use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::data_structures::KeyRole2;
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    PsqlOp,
};

use crate::utils::token_auth;
use crate::wallet::UpdateSubaccountHoldLimitRequest;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest, request_data: UpdateSubaccountHoldLimitRequest) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let UpdateSubaccountHoldLimitRequest {
        subaccount,
        limit,
    } = request_data;
    let main_account = super::get_main_account(user_id)?;
    super::have_no_uncompleted_tx(&main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

    //add wallet info
    let cli = ContractClient::<MultiSig>::new()?;    

    cli.update_subaccount_hold_limit(&main_account, &subaccount,limit).await?;

    Ok(None::<String>)
}
