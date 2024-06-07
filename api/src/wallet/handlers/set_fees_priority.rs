use actix_web::{web, HttpRequest};

use blockchain::{
    fees_call::FeesCall,
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, CoinType, KeyRole2},
    error_code::BackendError,
};
use models::{
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};
use tracing::debug;

use crate::utils::{get_user_context, token_auth};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetFeesPriorityRequest {
    fees_priority: Vec<String>,
}

pub async fn req(req: HttpRequest, request_data: SetFeesPriorityRequest) -> BackendRes<String> {
    //todo: must be called by main device
    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, token_version,device_id, device_brand) = token_auth::validate_credentials(&req,&mut db_cli).await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (main_account,current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole2::Master)?;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;

    let SetFeesPriorityRequest { fees_priority } = request_data;

    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new_update_cli().await?;

    if fees_priority.len() != 5 {
        Err(BackendError::RequestParamInvalid(
            "must specify 5 coin".to_string(),
        ))?;
    }

    let fees_priority = fees_priority
        .iter()
        .map(|x| {
            x.parse::<CoinType>()
                .map_err(|e| BackendError::RequestParamInvalid(e.to_string()))
        })
        .collect::<Result<Vec<CoinType>, BackendError>>()?;

    let tx_id = fees_call_cli
        .set_fees_priority(&main_account, fees_priority)
        .await?;
    debug!(
        "main_account {}: set_fees_priority txid {}",
        main_account, tx_id
    );

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::SetFeesPriority,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert(&mut db_cli).await?;
    Ok(None)
}
