use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use crate::wallet::TxListRequest;
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
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Mutex;

use super::ServentSigDetail;
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxViewTmp {
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
pub enum FilterType{
    OrderId,
    AccountId,
    Phone,
    Mail,
}
pub fn get_filter_type(data:&str) -> Result<FilterType,BackendError>{
    let wallet_suffix = & common::env::CONF.multi_sig_relayers[0].account_id;
    if data.contains('@') {
        Ok(FilterType::Mail)
    }else if data.contains('+'){
        Ok(FilterType::Phone)
    }else if data.contains(wallet_suffix){
        Ok(FilterType::AccountId)
    }else if  data.len() == 32{
        Ok(FilterType::OrderId)
    }else{
        Err(BackendError::RequestParamInvalid(data.to_string()))
    }
}

pub async fn req(req: HttpRequest, request_data: TxListRequest) -> BackendRes<Vec<CoinTxViewTmp>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
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
        match get_filter_type(&data)? {
            FilterType::OrderId => {
                CoinTxView::find(CoinTxFilter::ByOrderId(&data))
            },
            FilterType::AccountId => {
                CoinTxView::find(CoinTxFilter::ByTxRolePage(
                    tx_role,
                    &main_account,
                    Some(data),
                    per_page,
                    page,
                ))
            },
            FilterType::Phone | FilterType::Mail => {
                if let Ok(counterparty_main_account ) = UserInfoView::find_single(
                    UserFilter::ByPhoneOrEmail(&data)){
                        CoinTxView::find(CoinTxFilter::ByTxRolePage(
                            tx_role,
                            &main_account,
                            Some(&counterparty_main_account.user_info.main_account),
                            per_page,
                            page,
                        ))
                }else{
                    return Ok(None)
                }

            }
        }
    }else{
        CoinTxView::find( CoinTxFilter::ByTxRolePage(
            tx_role,
            &main_account,
            None,
            per_page,
            page,
        ))
    };
    
    
    
    let txs = find_res?;
    let txs: Vec<CoinTxViewTmp> = txs
        .into_iter()
        .map(|tx| -> Result<_> {
            let stage = if now_millis() > tx.transaction.expire_at {
                CoinSendStage::MultiSigExpired
            } else {
                tx.transaction.stage
            };

            Ok(CoinTxViewTmp {
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
                signatures: tx
                    .transaction
                    .signatures
                    .iter()
                    .map(|s| s.parse())
                    .collect::<Result<Vec<_>>>()?,
                tx_type: tx.transaction.tx_type,
                chain_status: tx.transaction.chain_status,
                updated_at: tx.updated_at,
                created_at: tx.created_at,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Some(txs))
}
