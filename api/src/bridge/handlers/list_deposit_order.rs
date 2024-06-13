use actix_web::HttpRequest;

use blockchain::bridge_on_near::Bridge;

use blockchain::ContractClient;
use common::data_structures::bridge::{DepositStatus, OrderType};

use models::eth_bridge_order::{BridgeOrderFilter, EthBridgeOrderEntity};

use models::PsqlOp;

use crate::utils::token_auth;
use crate::wallet::handlers::*;
use anyhow::Result;

use common::data_structures::CoinType;

use common::{error_code::BackendRes, utils::math::coin_amount::raw2display};
use serde::{Deserialize, Serialize};

use super::paginate_vec;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ListDepositOrderResponse {
    pub order_id: String,
    pub chain_id: u128,        //外链id
    pub account_id: String,    //无链id
    pub symbol: CoinType,      //代币符号
    pub amount: String,        //
    pub address: String,       //外链地址
    pub status: DepositStatus, //订单充值状态
    pub updated_at: String,    //更新时间
    pub created_at: String,    //创建时间
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListDepositOrderRequest {
    pub page: usize,
    pub per_page: usize,
}

//DRY
async fn list_chainless_order_ids(main_account: &str) -> Result<Vec<String>> {
    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;
    let orders = bridge_cli.list_deposit_order(main_account).await?;

    let orders = orders
        .unwrap_or(vec![])
        .into_iter()
        .filter(|(_id, info)| info.signature.is_some())
        .map(|(id, _)| id.to_string())
        .collect();
    Ok(orders)
}

pub async fn list_external_orders(main_account: &str) -> Result<Vec<EthBridgeOrderEntity>> {
    EthBridgeOrderEntity::find(BridgeOrderFilter::ByTypeAndAccountId(
        OrderType::Deposit,
        main_account,
    ))
    .await
}

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ListDepositOrderRequest,
) -> BackendRes<Vec<ListDepositOrderResponse>> {
    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    //todo:
    let main_account = get_main_account(user_id).await?;

    let ListDepositOrderRequest {
        page,
        per_page: page_size,
    } = request_data;

    let order_ids_on_chainless = list_chainless_order_ids(&main_account).await?;
    let orders_on_external = list_external_orders(&main_account).await?;

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
