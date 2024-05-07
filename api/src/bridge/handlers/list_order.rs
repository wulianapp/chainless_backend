use std::collections::BTreeMap;

use actix_web::HttpRequest;

use blockchain::bridge_on_near::Bridge;
use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};
use blockchain::ContractClient;
use common::utils::time::timestamp2utc;

use crate::bridge::{ListWithdrawOrderResponse, SignedOrderResponse};
use crate::{utils::token_auth, wallet::MultiSigRankExternal};
use common::error_code::BackendError::ChainError;
use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use serde::{Deserialize, Serialize};
use crate::wallet::handlers::*;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRankTmp>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: BTreeMap<String, SubAccConf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRankTmp {
    min: String,
    max_eq: String,
    sig_num: u8,
}

pub(crate) async fn req(req: HttpRequest) -> BackendRes<Vec<ListWithdrawOrderResponse>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let main_account = get_main_account(user_id)?;
    let bridge_cli = ContractClient::<Bridge>::new()?;

    let orders = bridge_cli.list_withdraw_order(&main_account).await?;
    let orders = orders
    .unwrap_or((0,vec![]))
    .1
    .into_iter()
    .map(|(id,info)| {

        let signers = 
        info.signers
        .into_iter()
        .map(|sig|{
            SignedOrderResponse{
                number: sig.number,
                signer: sig.signer.to_string(),
                signature: sig.signature,
            }
        }).collect();

        Ok(ListWithdrawOrderResponse{
            order_id: id,
            chain_id: info.chain_id,
            order_type: format!("{:?}",info.order_type),
            account_id: info.account_id.to_string(),
            symbol: info.symbol.parse()?,
            amount: raw2display(info.amount),
            address: info.address,
            signers: signers,
            signature: info.signature,
            status: format!("{:?}",info.status),
            updated_at: timestamp2utc(info.update_at),
            created_at: timestamp2utc(info.create_at),
        })
    }).collect::<Result<Vec<_>>>()?;
    Ok(Some(orders))
}
