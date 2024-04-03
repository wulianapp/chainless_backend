use std::collections::HashMap;

use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};

use crate::utils::token_auth;
use common::error_code::{BackendRes, WalletError};
use serde::{Deserialize, Serialize};
use common::error_code::BackendError::ChainError;
use crate::wallet::GetNeedSigNumRequest;



pub(crate) async fn req(req: HttpRequest,request_data: GetNeedSigNumRequest) -> BackendRes<u8> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;
    let GetNeedSigNumRequest{coin,amount} = request_data;

    let strategy = multi_cli.get_strategy(&main_account).await?;
    if strategy.is_none(){
        Err(WalletError::NotSetSecurity)?;
    }

    let coin_type = coin.parse().unwrap();
    let amount = amount.parse().unwrap();
    let need_sig_num = super::get_servant_need(
        &strategy.unwrap().multi_sig_ranks,
        &coin_type,
        amount,
    ).await; 
    Ok(Some(need_sig_num))
}
