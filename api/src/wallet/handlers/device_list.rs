use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, StrategyData};
use common::data_structures::{KeyRole, KeyRole2};
use models::general::get_pg_pool_connect;

use crate::utils::token_auth;
use blockchain::ContractClient;
use common::data_structures::device_info::DeviceInfo;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use std::cmp::Ordering;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<DeviceInfo>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;
    let devices: Vec<DeviceInfoView> = DeviceInfoView::find(DeviceInfoFilter::ByUser(user_id),&mut pg_cli).await?;
    let mut devices: Vec<DeviceInfo> = devices.into_iter().map(|x| x.device_info).collect();
    //order by master <- servant <- undefined
    devices.sort_by(|a, b| {
        if a.key_role == KeyRole2::Master
            || (a.key_role == KeyRole2::Servant && b.key_role == KeyRole2::Undefined)
        {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    Ok(Some(devices))
}
