use actix_web::HttpRequest;
use blockchain::air_reward::AirReward;
use blockchain::ContractClient;
use common::data_structures::{KeyRole2, OpStatus};
use common::error_code::BackendRes;

use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::get_pg_pool_connect;
use models::{account_manager, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;
//use super::super::ContactIsUsedRequest;
use crate::air_reward::SysInfoResponse;
use crate::utils::token_auth;


pub async fn req(req: HttpRequest) -> BackendRes<SysInfoResponse> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli: PgLocalCli = get_pg_pool_connect().await?;

    let _devices =
        DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id),&mut pg_cli).await?;
    let _res = account_manager::UserInfoView::find_single(UserFilter::ById(user_id),&mut pg_cli).await?;

    //todo:
    /*** 
    let role = if res.user_info.main_account.eq("") {
        KeyRole2::Undefined
    } else {
        let (_, current_strategy, device) =
            crate::wallet::handlers::get_session_state(user_id, &device_id,&mut pg_cli).await?;
        let current_role =
            crate::wallet::handlers::get_role(&current_strategy, device.hold_pubkey.as_deref());
        current_role
    };
    **/
    let cli = ContractClient::<AirReward>::new().await.unwrap();
    let sys_info = cli.get_sys_info().await?;
    Ok(sys_info)
}
