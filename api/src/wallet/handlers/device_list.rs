use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, StrategyData};
use common::data_structures::KeyRole2;
use models::general::get_pg_pool_connect;

use crate::utils::{get_user_context, judge_role_by_strategy, token_auth};
use blockchain::ContractClient;
use common::data_structures::device_info::DeviceInfo;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::PsqlOp;
use std::cmp::Ordering;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<DeviceInfo>> {
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;

    let mut db_cli = get_pg_pool_connect().await?;
    let mut devices: Vec<DeviceInfo> = DeviceInfoEntity::find(
            DeviceInfoFilter::ByUser(&user_id),
             &mut db_cli
            ).await?
            .into_iter()
            .map(|d| d.into_inner())
            .collect();

    let context = get_user_context(&user_id,&device_id,&mut db_cli).await?;
    
    devices.sort_by(|a, b| {
        let a_role = judge_role_by_strategy(context.strategy.as_ref(),a.hold_pubkey.as_deref()).unwrap();
        let b_role = judge_role_by_strategy(context.strategy.as_ref(),b.hold_pubkey.as_deref()).unwrap();
        if a_role == KeyRole2::Master
            || (a_role == KeyRole2::Servant && b_role == KeyRole2::Undefined)
        {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });        
    Ok(Some(devices))
}
