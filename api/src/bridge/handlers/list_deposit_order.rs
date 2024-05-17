use std::collections::BTreeMap;

use actix_web::HttpRequest;

use blockchain::bridge_on_near::{Bridge, BridgeOrder};
use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::bridge::{DepositStatus, OrderType};
use common::utils::time::timestamp2utc;
use models::eth_bridge_order::{BridgeOrderFilter, EthBridgeOrderView};
use models::PsqlOp;

use crate::bridge::{ListDepositOrderRequest, ListDepositOrderResponse, ListWithdrawOrderResponse};
use crate::wallet::handlers::*;
use crate::{utils::token_auth, wallet::MultiSigRankExternal};
use anyhow::Result;
use common::data_structures::bridge::EthOrderStatus;
use common::error_code::BackendError::ChainError;
use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use serde::{Deserialize, Serialize};

use super::paginate_vec;

//DRY
async fn list_chainless_order_ids(main_account: &str) -> Result<Vec<String>> {
    let bridge_cli = ContractClient::<Bridge>::new().await?;
    let orders = bridge_cli.list_deposit_order(main_account).await?;

    let orders = orders
        .unwrap_or(vec![])
        .into_iter()
        .filter(|(_id, info)| info.signature.is_some())
        .map(|(id, _)| id.to_string())
        .collect();
    Ok(orders)
}

pub fn list_external_orders(main_account: &str) -> Result<Vec<EthBridgeOrderView>> {
    EthBridgeOrderView::find(BridgeOrderFilter::ByTypeAndAccountId(
        OrderType::Deposit,
        main_account,
    ))
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ListDepositOrderRequest,
) -> BackendRes<Vec<ListDepositOrderResponse>> {
    let user_id = token_auth::validate_credentials(&req)?;
    //todo:
    let main_account = get_main_account(user_id)?;

    let ListDepositOrderRequest {
        page,
        per_page: page_size,
    } = request_data;

    let order_ids_on_chainless = list_chainless_order_ids(&main_account).await?;
    let orders_on_external = list_external_orders(&main_account)?;

    let mut all_order = vec![];
    for order in orders_on_external {
        let mut status = order.order.status.into();
        if order_ids_on_chainless.contains(&order.order.id) {
            status = DepositStatus::ChainLessSuccessful
        }

        all_order.push(ListDepositOrderResponse {
            order_id: order.order.id,
            chain_id: 1500,
            account_id: order.order.chainless_acc,
            symbol: order.order.coin,
            amount: raw2display(order.order.amount),
            address: order.order.eth_addr,
            status,
            updated_at: order.updated_at,
            created_at: order.created_at,
        })
    }
    let page_order = paginate_vec(all_order, page_size, page);
    Ok(Some(page_order))
}
