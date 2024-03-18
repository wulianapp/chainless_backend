use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{get_support_coin_list, CoinTransaction, CoinTxStatus, CoinType};
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::coin_transfer::{CoinTxFilter, CoinTxView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use crate::wallet::TxListRequest;


#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxViewTmp {
    pub tx_index: u32,
    pub tx_id: Option<String>,
    pub coin_type: CoinType,
    pub from: String, //uid
    pub to: String,   //uid
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub status: CoinTxStatus,
    pub coin_tx_raw: String,
    pub chain_tx_raw: Option<String>,
    pub signatures: Vec<String>,
    pub updated_at: String,
    pub created_at: String,
}


pub async fn req(req: HttpRequest,request_data: TxListRequest) -> BackendRes<Vec<CoinTxViewTmp>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let TxListRequest{ tx_role, counterparty, per_page, page } = request_data;
    let tx_role = tx_role.parse().unwrap();
    let main_account = user_info.user_info.main_account;
    let txs = CoinTxView::find(CoinTxFilter::ByTxRolePage(tx_role,&main_account,counterparty.as_deref(),per_page,page))?;
    let txs = txs.into_iter().map(|tx|{
        CoinTxViewTmp{
            tx_index: tx.tx_index,
            tx_id: tx.transaction.tx_id,
            coin_type: tx.transaction.coin_type,
            from: tx.transaction.from,
            to: tx.transaction.to,
            amount: tx.transaction.amount,
            expire_at: tx.transaction.expire_at,
            memo: tx.transaction.memo,
            status: tx.transaction.status,
            coin_tx_raw: tx.transaction.coin_tx_raw,
            chain_tx_raw: tx.transaction.chain_tx_raw,
            signatures: tx.transaction.signatures,
            updated_at: tx.updated_at,
            created_at: tx.created_at,
        }
    }).collect();
   
    Ok(Some(txs))
}
