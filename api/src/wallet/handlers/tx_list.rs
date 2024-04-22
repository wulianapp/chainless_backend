use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::{get_support_coin_list, coin_transaction::{CoinTransaction,TxType}, CoinType};
use common::data_structures::TxStatusOnChain;
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::utils::math::coin_amount::raw2display;
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Mutex;
use crate::wallet::TxListRequest;

use super::ServentSigDetail;


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


pub async fn req(req: HttpRequest,request_data: TxListRequest) -> BackendRes<Vec<CoinTxViewTmp>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let TxListRequest{ tx_role, counterparty, per_page, page } = request_data;
    let tx_role = tx_role.parse().unwrap();
    let txs = CoinTxView::find(CoinTxFilter::ByTxRolePage(tx_role,&main_account,counterparty.as_deref(),per_page,page))?;
    let txs: Vec<CoinTxViewTmp> = txs.into_iter().map(|tx|{
        CoinTxViewTmp{
            order_id: tx.transaction.order_id,
            tx_id: tx.transaction.tx_id,
            coin_type: tx.transaction.coin_type,
            from: tx.transaction.from,
            to: tx.transaction.to,
            amount: raw2display(tx.transaction.amount),
            expire_at: tx.transaction.expire_at,
            memo: tx.transaction.memo,
            stage: tx.transaction.stage,
            coin_tx_raw: tx.transaction.coin_tx_raw,
            chain_tx_raw: tx.transaction.chain_tx_raw,
            //todo
            signatures: tx.transaction.signatures.iter().map(|s| s.parse().unwrap()).collect(),
            tx_type: tx.transaction.tx_type,
            chain_status: tx.transaction.chain_status,
            updated_at: tx.updated_at,
            created_at: tx.created_at,
        }
    }).collect();
   
    Ok(Some(txs))
}
