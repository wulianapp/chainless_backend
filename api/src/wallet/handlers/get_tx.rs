use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, FeesDetailResponse};
use crate::wallet::{GetTxRequest, GetTxResponse};
use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::KeyRole2;
use common::data_structures::{
    coin_transaction::{CoinSendStage, CoinTransaction, TxType},
    get_support_coin_list, CoinType, TxStatusOnChain,
};
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::error_code::{BackendError, WalletError};
use common::utils::math::coin_amount::raw2display;
use common::utils::math::hex_to_bs58;
use common::utils::time::now_millis;
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Mutex;
use tracing::error;

use super::ServentSigDetail;
use blockchain::fees_call::*;

//todo: txs 放在上层
async fn get_actual_fee(account_id: &str, dist_tx_id: &str) -> Result<Vec<(CoinType, u128)>> {
    let fees_call_cli = blockchain::ContractClient::<FeesCall>::new()?;
    let txs = fees_call_cli.get_user_txs(account_id).await?;
    for (index, (fee_id, amount, tx_id, _memo)) in txs.iter().enumerate() {
        if let Some(id) = tx_id {
            if id == hex_to_bs58(dist_tx_id)?.as_str() {
                let gas_fee_token = fee_id.parse()?;
                let gas_fee_amount = amount;
                let protocol_fee_token = txs[index + 1].0.parse()?;
                let protocol_fee_amount = txs[index + 1].1;

                if gas_fee_token == protocol_fee_token {
                    return Ok(vec![(gas_fee_token, gas_fee_amount + protocol_fee_amount)]);
                } else {
                    return Ok(vec![
                        (gas_fee_token, *gas_fee_amount),
                        (protocol_fee_token, protocol_fee_amount),
                    ]);
                }
            }
        }
    }
    Err(anyhow!("tx_id: {} not found on fees_call", dist_tx_id))
}

pub async fn req(req: HttpRequest, request_data: GetTxRequest) -> BackendRes<GetTxResponse> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let GetTxRequest { order_id } = request_data;
    let tx = CoinTxView::find_single(CoinTxFilter::ByOrderId(&order_id)).map_err(|e| {
        if e.to_string().contains("DBError::DataNotFound") {
            WalletError::OrderNotFound(order_id).into()
        } else {
            BackendError::InternalError(e.to_string())
        }
    })?;

    let signed_device: Vec<ServentSigDetail> = tx
        .transaction
        .signatures
        .iter()
        .map(|s| s.parse())
        .collect::<Result<_>>()?;

    //不从数据库去读
    let all_device = DeviceInfoView::find(DeviceInfoFilter::ByUser(user_id))?
        .into_iter()
        .filter(|x| {
            x.device_info.hold_pubkey.is_some() && x.device_info.key_role == KeyRole2::Servant
        })
        .map(|d| ServentSigDetail {
            pubkey: d.device_info.hold_pubkey.unwrap(),
            device_id: d.device_info.id,
            device_brand: d.device_info.brand,
        })
        .collect::<Vec<ServentSigDetail>>();

    let unsigned_device = all_device
        .into_iter()
        .filter(|x| !signed_device.contains(x))
        .collect();

    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.to_owned()))?;

    let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &tx.transaction.coin_type,
        tx.transaction.amount,
    )
    .await;

    let fees_detail = if tx.transaction.chain_status == TxStatusOnChain::Successful {
        let tx_id = tx.transaction.tx_id.as_ref().ok_or("")?;
        get_actual_fee(&tx.transaction.from, tx_id)
            .await?
            .into_iter()
            .map(|(fee_coin, amount)| FeesDetailResponse {
                fee_coin,
                fee_amount: raw2display(amount),
            })
            .collect()
    } else {
        let (fee_coin, fee_amount, _balance_enough) = super::estimate_transfer_fee(
            &tx.transaction.from,
            &tx.transaction.coin_type,
            tx.transaction.amount,
        )
        .await?;
        vec![FeesDetailResponse {
            fee_coin,
            fee_amount: raw2display(fee_amount),
        }]
    };
    let stage = if now_millis() > tx.transaction.expire_at {
        CoinSendStage::MultiSigExpired
    } else {
        tx.transaction.stage
    };
    let tx = GetTxResponse {
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
        need_sig_num,
        signed_device,
        unsigned_device,
        tx_type: tx.transaction.tx_type,
        chain_status: tx.transaction.chain_status,
        //fee_coin,
        //fee_amount: raw2display(fee_amount),
        fees_detail,
        updated_at: tx.updated_at,
        created_at: tx.created_at,
    };
    Ok(Some(tx))
}
