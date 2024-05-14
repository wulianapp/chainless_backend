use actix_web::{web, HttpRequest};

use blockchain::{air_reward::AirReward, multi_sig::{MultiSig, MultiSigRank}};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{BackendError,AccountManagerError},
    utils::math::coin_amount::display2raw,
};
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    wallet_manage_record::WalletManageRecordView,
    PsqlOp,
};
use tracing::{debug, info};

use crate::{air_reward::ReceiveAirRequest, utils::{token_auth, wallet_grades::query_wallet_grade}};
use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use crate::wallet::handlers::*;

pub async fn req(req: HttpRequest, request_data: ReceiveAirRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;

    let (user, current_strategy, device) = get_session_state(user_id, &device_id).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id)?;


    let ReceiveAirRequest {
        btc_addr,
        sig
    } = request_data;

    let btc_addr_level = match (btc_addr,sig) {
        (None, None) => {
            None
        },
        (Some(addr), Some(_)) => {
            //todo: check sig
            let grade = query_wallet_grade(&addr).await?;
            Some((addr,grade))
        },
        _ => Err(BackendError::RequestParamInvalid("".to_string()))?
    };

    let is_real = Some(false);

    let cli = ContractClient::<AirReward>::new()?;
    let ref_user = cli.get_up_user_with_id(&main_account)
    .await?
    .ok_or(AccountManagerError::PredecessorNotSetSecurity)?;

    let receive_res = cli.receive_air(
        &main_account,
        &ref_user.user_account.to_string(),
        btc_addr_level,
        is_real
    ).await?;
    debug!("successful claim air_reward {:?}",receive_res);
    Ok(None)
}
