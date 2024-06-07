use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole},
    error_code::BackendError,
    utils::math::coin_amount::display2raw,
};
use models::{
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};

use crate::utils::{get_user_context, token_auth};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use serde::{Deserialize, Serialize};

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

    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (main_account, current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;

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
        .map_err(|_e| WalletError::UnSupportedPrecision)?;
    //add wallet info
    let cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let tx_id = cli.update_rank(&main_account, strategy).await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::UpdateStrategy,
        &context.device.hold_pubkey.unwrap(),
        &context.device.id,
        &context.device.brand,
        vec![tx_id],
    );
    record.insert(&mut db_cli).await?;

    Ok(None::<String>)
}
