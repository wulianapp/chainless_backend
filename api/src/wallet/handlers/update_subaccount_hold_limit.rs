use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::BackendError,
    utils::math::coin_amount::display2raw,
};
use models::{
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};

use crate::utils::token_auth;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubaccountHoldLimitRequest {
    subaccount: String,
    limit: String,
}

pub async fn req(
    req: HttpRequest,
    request_data: UpdateSubaccountHoldLimitRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let UpdateSubaccountHoldLimitRequest { subaccount, limit } = request_data;
    let limit = display2raw(&limit).map_err(|_e| WalletError::UnSupportedPrecision)?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new().await?;

    let txid = cli
        .update_subaccount_hold_limit(&main_account, &subaccount, limit)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::UpdateSubaccountHoldLimit,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert(&mut db_cli).await?;
    Ok(None::<String>)
}
