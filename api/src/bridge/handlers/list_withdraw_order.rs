use std::collections::BTreeMap;

use actix_web::HttpRequest;

use blockchain::bridge_on_near::Status as StatusOnNear;
use blockchain::bridge_on_near::{Bridge, BridgeOrder};
use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::bridge::{OrderType, WithdrawStatus};
use common::error_code::parse_str;
use common::utils::time::timestamp2utc;
use models::eth_bridge_order::{BridgeOrderFilter, EthBridgeOrderEntity};
use models::PsqlOp;
use tracing_subscriber::filter;

use crate::utils::token_auth;
use crate::wallet::handlers::*;
use anyhow::Result;
use common::data_structures::bridge::EthOrderStatus;
use common::data_structures::CoinType;
use common::error_code::BackendError::ChainError;
use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use models::general::get_pg_pool_connect;
use serde::{Deserialize, Serialize};

use super::paginate_vec;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ListWithdrawOrderResponse {
    pub order_id: String,
    pub chain_id: u128,     //外链id
    pub account_id: String, //无链id
    pub symbol: CoinType,   //代币符号
    pub amount: String,
    pub address: String,         //外链地址
    pub signatures: Vec<String>, //签名详情
    pub status: WithdrawStatus,  //订单提现状态
    pub created_at: String,      //创建时间
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWithdrawOrderRequest {
    pub page: usize,
    pub per_page: usize,
}

pub async fn list_chainless_orders(main_account: &str) -> Result<Vec<(u128, BridgeOrder)>> {
    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;
    let orders = bridge_cli.list_withdraw_order(main_account).await?;

    let orders = orders
        .unwrap_or(vec![])
        .into_iter()
        .filter(|(_id, info)| info.signature.is_none())
        .collect();
    Ok(orders)
}

pub async fn list_external_orders(main_account: &str) -> Result<Vec<EthBridgeOrderEntity>> {
    let mut db_cli = get_pg_pool_connect().await?;
    EthBridgeOrderEntity::find(
        BridgeOrderFilter::ByTypeAndAccountId(OrderType::Withdraw, main_account),
        &mut db_cli,
    )
    .await
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ListWithdrawOrderRequest,
) -> BackendRes<Vec<ListWithdrawOrderResponse>> {
    let mut db_cli = get_pg_pool_connect().await?;

    let (user_id, _, _, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;
    let main_account = get_main_account(user_id, &mut db_cli).await?;

    let ListWithdrawOrderRequest {
        page,
        per_page: page_size,
    } = request_data;

    let mut order_ids_on_chainless = list_chainless_orders(&main_account).await?;
    order_ids_on_chainless.reverse();
    let orders_on_external = list_external_orders(&main_account).await?;
    let mut all_order = vec![];
    for (id, info) in order_ids_on_chainless {
        let status = match info.status {
            StatusOnNear::Syncless => WithdrawStatus::ChainLessSigning,
            StatusOnNear::Pending => WithdrawStatus::ChainLessSigning,
            StatusOnNear::Default => WithdrawStatus::ChainLessSigning,
            StatusOnNear::Signed | StatusOnNear::Completed => {
                let external_order: Vec<&EthBridgeOrderEntity> = orders_on_external
                    .iter()
                    .filter(|x| x.order.id == id.to_string())
                    .collect();
                if external_order.is_empty() {
                    WithdrawStatus::ChainLessSuccessful
                } else if external_order.len() > 1 {
                    panic!("internal error");
                } else {
                    external_order[0].order.status.to_owned().into()
                }
            }
        };
        //Signed的情况下，签名的时候不会有位None的情况，非Signed的情况下也不用关注
        let signatures = info
            .signers
            .into_iter()
            .filter_map(|x| x.signature)
            .collect();
        all_order.push(ListWithdrawOrderResponse {
            order_id: id.to_string(),
            chain_id: 1500,
            account_id: info.account_id.to_string(),
            symbol: parse_str(info.symbol)?,
            amount: raw2display(info.amount),
            address: info.address,
            status,
            signatures,
            //updated_at: timestamp2utc(info.update_at),
            created_at: timestamp2utc(info.create_at),
        })
    }
    let page_order = paginate_vec(all_order, page_size, page);
    Ok(Some(page_order))
}
