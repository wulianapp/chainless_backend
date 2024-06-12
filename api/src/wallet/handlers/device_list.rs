use actix_web::HttpRequest;


use common::data_structures::{DeviceState, KeyRole};

use serde::{Deserialize, Serialize};

use crate::utils::{get_user_context, judge_role_by_strategy, token_auth};

use common::data_structures::device_info::DeviceInfo;
use common::error_code::BackendRes;

use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::PsqlOp;
use std::cmp::Ordering;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct DeviceListResponse {
    pub id: String,
    pub user_id: u32,
    pub state: DeviceState,
    pub hold_pubkey: Option<String>,
    pub brand: String,
    pub holder_confirm_saved: bool,
    pub key_role: KeyRole,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<DeviceListResponse>> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let devices: Vec<DeviceInfo> = DeviceInfoEntity::find(DeviceInfoFilter::ByUser(&user_id))
        .await?
        .into_iter()
        .map(|d| d.into_inner())
        .collect();

    let context = get_user_context(&user_id, &device_id).await?;

    let mut devices_res: Vec<DeviceListResponse> = devices
        .into_iter()
        .map(|device| {
            let role =
                judge_role_by_strategy(context.strategy.as_ref(), device.hold_pubkey.as_deref())
                    .unwrap();
            DeviceListResponse {
                id: device.id,
                user_id: device.user_id,
                state: device.state,
                hold_pubkey: device.hold_pubkey,
                brand: device.brand,
                holder_confirm_saved: device.holder_confirm_saved,
                key_role: role,
            }
        })
        .collect();

    devices_res.sort_by(|a, b| {
        if a.key_role == KeyRole::Master
            || (a.key_role == KeyRole::Servant && b.key_role == KeyRole::Undefined)
        {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    Ok(Some(devices_res))
}
