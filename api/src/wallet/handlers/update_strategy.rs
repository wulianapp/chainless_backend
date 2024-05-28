use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::BackendError,
    utils::math::coin_amount::display2raw,
};
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordView,
    PsqlOp,
};

use crate::utils::token_auth;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use serde::{Deserialize,Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigRankRequest {
    min: String,
    max_eq: String,
    sig_num: u8,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategyRequest {
    strategy: Vec<MultiSigRankRequest>,
}


pub async fn req(req: HttpRequest, request_data: UpdateStrategyRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let UpdateStrategyRequest { strategy } = request_data;
    if strategy.len() > current_strategy.servant_pubkeys.len() + 1 {
        Err(WalletError::StrategyRankIllegal)?;
    }

    //fixme:
    let strategy = strategy
        .into_iter()
        .map(|x| -> Result<MultiSigRank, String> {
            let rank = MultiSigRank {
                min: display2raw(&x.min)?,
                max_eq: display2raw(&x.max_eq)?,
                sig_num: x.sig_num,
            };
            Ok(rank)
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(BackendError::RequestParamInvalid)?;
    //add wallet info
    let cli = ContractClient::<MultiSig>::new().await?;
    let tx_id = cli.update_rank(&main_account, strategy).await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::UpdateStrategy,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![tx_id],
    );
    record.insert(&mut db_cli).await?;

    Ok(None::<String>)
}
