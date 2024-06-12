use actix_web::{HttpRequest};

use blockchain::multi_sig::{MultiSig};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole},
    utils::math::coin_amount::display2raw,
};
use models::{
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};

use crate::utils::{get_user_context, token_auth};
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
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let UpdateSubaccountHoldLimitRequest { subaccount, limit } = request_data;
    let limit = display2raw(&limit).map_err(|_e| WalletError::UnSupportedPrecision)?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new_update_cli().await?;

    let txid = cli
        .update_subaccount_hold_limit(&main_account, &subaccount, limit)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::UpdateSubaccountHoldLimit,
        &context.device.hold_pubkey.unwrap(),
        &context.device.id,
        &context.device.brand,
        vec![txid],
    );
    record.insert().await?;
    Ok(None::<String>)
}
