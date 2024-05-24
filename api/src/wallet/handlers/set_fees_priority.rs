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
    device_info::{DeviceInfoFilter, DeviceInfoView}, general::get_pg_pool_connect, wallet_manage_record::WalletManageRecordView, PsqlOp
};
use tracing::debug;

use crate::utils::token_auth;
use crate::wallet::SetFeesPriorityRequest;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<SetFeesPriorityRequest>,
) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;
    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id,&mut pg_cli).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account,&mut pg_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let SetFeesPriorityRequest { fees_priority } = request_data.0;

    let main_account = super::get_main_account(user_id,&mut pg_cli).await?;
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new().await?;

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
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::SetFeesPriority,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert(&mut pg_cli).await?;
    Ok(None)
}
