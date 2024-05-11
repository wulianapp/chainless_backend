use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, StrategyData};
use common::data_structures::{KeyRole, KeyRole2};

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
    /***
    let mut all_keys = vec![];
    let find_res = UserInfoView::find_single(UserFilter::ById(user_id))?;
    if find_res.user_info.main_account != ""{
        //todo: if change master key, main_account not equal anymore
        all_keys.push(find_res.user_info.main_account.clone());
        let cli = ContractClient::<MultiSig>::new();
        let mut res = cli.get_strategy(&find_res.user_info.main_account).await?;
        if let Some(mut strategy) = res {
            all_keys.append(&mut strategy.servant_pubkeys);
        }
    }
    */
    let devices: Vec<DeviceInfoView> = DeviceInfoView::find(DeviceInfoFilter::ByUser(user_id))?;

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
