use crate::utils::token_auth;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::TxStatusOnChain;
use common::data_structures::{
    coin_transaction::{CoinTransaction, TxType},
    get_support_coin_list, CoinType,
};
use common::error_code::to_param_invalid_error;
use common::error_code::AccountManagerError;
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::utils::math::coin_amount::raw2display;
use common::utils::time::now_millis;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::coin_transfer::{CoinTxEntity, CoinTxFilter};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Mutex;

use super::ServentSigDetail;
use anyhow::Result;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxListRequest {
    tx_role: String,
    counterparty: Option<String>,
    per_page: u32,
    page: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxViewResponse {
    pub order_id: String,
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: String,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub stage: CoinSendStage,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<ServentSigDetail>,
    pub tx_type: TxType,
    pub chain_status: TxStatusOnChain,
    pub updated_at: String,
    pub created_at: String,
}
pub enum FilterType {
    OrderId,
    AccountId,
    Phone,
    Mail,
}
pub fn get_filter_type(data: &str) -> Result<FilterType, BackendError> {
    let wallet_suffix = &common::env::CONF.relayer_pool.base_account_id;
    if data.contains('@') {
        Ok(FilterType::Mail)
    } else if data.contains('+') {
        Ok(FilterType::Phone)
    } else if data.contains(wallet_suffix) {
        Ok(FilterType::AccountId)
    } else if data.len() == 32 {
        Ok(FilterType::OrderId)
    } else {
        Err(BackendError::RequestParamInvalid(data.to_string()))
    }
}

pub async fn req(
    req: HttpRequest,
    request_data: TxListRequest,
) -> BackendRes<Vec<CoinTxViewResponse>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let main_account = super::get_main_account(user_id, &mut db_cli).await?;
    let TxListRequest {
        tx_role,
        counterparty,
        per_page,
        page,
    } = request_data;
    if per_page > 1000 {
        Err(BackendError::RequestParamInvalid(
            "per_page is too big".to_string(),
        ))?;
    }
    let tx_role = tx_role.parse().map_err(to_param_invalid_error)?;

    //filter by tx_order_id 、account_id 、phone、mail or eth_addr
    //fixme:
    let find_res = if let Some(data) = counterparty.as_deref() {
        match get_filter_type(data)? {
            FilterType::OrderId => {
                CoinTxEntity::find(CoinTxFilter::ByOrderId(data), &mut db_cli).await
            }
            FilterType::AccountId => {
                CoinTxEntity::find(
                    CoinTxFilter::ByTxRolePage(tx_role, &main_account, Some(data), per_page, page),
                    &mut db_cli,
                )
                .await
            }
            FilterType::Phone | FilterType::Mail => {
                if let Ok(counterparty_main_account) =
                    UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(data), &mut db_cli).await
                {
                    CoinTxEntity::find(
                        CoinTxFilter::ByTxRolePage(
                            tx_role,
                            &main_account,
                            Some(&counterparty_main_account.user_info.main_account),
                            per_page,
                            page,
                        ),
                        &mut db_cli,
                    )
                    .await
                } else {
                    return Ok(None);
                }
            }
        }
    } else {
        CoinTxEntity::find(
            CoinTxFilter::ByTxRolePage(tx_role, &main_account, None, per_page, page),
            &mut db_cli,
        )
        .await
    };

    let txs = find_res?;

    let mut view_txs = vec![];
    for tx in txs {
        let stage = if now_millis() > tx.transaction.expire_at {
            CoinSendStage::MultiSigExpired
        } else {
            tx.transaction.stage
        };
        let mut sigs = vec![];
        for sig in tx.transaction.signatures {
            let pubkey = sig[..64].to_string();
            let device =
                DeviceInfoEntity::find_single(DeviceInfoFilter::ByHoldKey(&pubkey), &mut db_cli)
                    .await?;
            let sig = ServentSigDetail {
                pubkey,
                device_id: device.device_info.id,
                device_brand: device.device_info.brand,
            };
            sigs.push(sig);
        }
        view_txs.push(CoinTxViewResponse {
            order_id: tx.transaction.order_id,
            tx_id: tx.transaction.tx_id,
            coin_type: tx.transaction.coin_type,
            from: tx.transaction.from,
            to: tx.transaction.to,
            amount: raw2display(tx.transaction.amount),
            expire_at: tx.transaction.expire_at,
            memo: tx.transaction.memo,
            stage,
            coin_tx_raw: tx.transaction.coin_tx_raw,
            chain_tx_raw: tx.transaction.chain_tx_raw,
            signatures: sigs,
            tx_type: tx.transaction.tx_type,
            chain_status: tx.transaction.chain_status,
            updated_at: tx.updated_at,
            created_at: tx.created_at,
        });
    }
    Ok(Some(view_txs))
}
