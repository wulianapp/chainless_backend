use std::time;

use anyhow::Ok;
use anyhow::Result;
use blockchain::bridge_on_eth::Bridge;
use blockchain::eth_cli::general::*;
use blockchain::eth_cli::EthContractClient;
use common::constants::ETH_TX_CONFIRM_BLOCK_NUM;
use common::data_structures::bridge::EthOrderStatus;
use common::data_structures::bridge::OrderType;
use models::eth_bridge_order::BridgeOrderUpdater;
use models::eth_bridge_order::{BridgeOrderFilter, EthBridgeOrderEntity};

use models::PsqlOp;
use tracing::info;

//如果没历史监控数据，则从固定检查点开始扫,如果有则从历史数据中的最后高度开始扫
pub async fn get_last_process_height() -> Result<u64> {
    let last_order = EthBridgeOrderEntity::find(BridgeOrderFilter::Limit(1)).await?;
    if last_order.is_empty() {
        //Ok(get_current_block().await)
        Ok(1446063)
    } else {
        //Ok(1322262)
        Ok(last_order[0].order.height)
    }
}

//listen and then insert pending
pub async fn listen_newest_block(bridge: &EthContractClient<Bridge>, height: u64) -> Result<()> {
    let block_hash = get_block(height).await?.unwrap().hash.unwrap();
    let block_hash = hex::encode(block_hash.as_bytes());

    let deposit_orders = bridge.filter_deposit_event(&block_hash).await.unwrap();
    if deposit_orders.is_empty() {
        //info!("Not found deposit0 orders created at height {}",height);
    } else {
        info!(
            "filter_deposit_event {:?} at height {}",
            deposit_orders, height
        );
        //todo: batch insert
        for order in deposit_orders {
            let order = EthBridgeOrderEntity::new_with_specified(
                &order.id,
                &order.chainless_acc,
                &order.eth_addr,
                order.order_type,
                order.coin,
                order.amount,
                EthOrderStatus::Pending,
                height,
            );
            order.insert().await?;
        }
    }

    let withdraw_orders = bridge.filter_withdraw_event(&block_hash).await.unwrap();

    if withdraw_orders.is_empty() {
        //info!("Not found new filter_withdraw_event3 created at height {}", height);
    } else {
        info!(
            "filter_withdraw_event {:?} at height {}",
            withdraw_orders, height
        );
        for order in withdraw_orders {
            let order = EthBridgeOrderEntity::new_with_specified(
                &order.id,
                &order.chainless_acc,
                &order.eth_addr,
                order.order_type,
                order.coin,
                order.amount,
                EthOrderStatus::Confirmed,
                height,
            );
            order.insert().await?;
        }
    }
    Ok(())
}

//listen and then update to confirm
//DRY
pub async fn listen_confirmed_block(bridge: &EthContractClient<Bridge>, height: u64) -> Result<()> {
    let block_hash = get_block(height).await?.unwrap().hash.unwrap();
    let block_hash = hex::encode(block_hash.as_bytes());
    //info!("check block_hash {}", block_hash);

    let deposit_orders = bridge.filter_deposit_event(&block_hash).await.unwrap();
    if deposit_orders.is_empty() {
        //info!("Not found deposit0 orders created at height {}",height);
    } else {
        info!(
            "filter_deposit_event {:?} at height {}",
            deposit_orders, height
        );
        //todo: batch insert
        for order in deposit_orders {
            EthBridgeOrderEntity::update_single(
                BridgeOrderUpdater::Status(EthOrderStatus::Confirmed),
                BridgeOrderFilter::ByTypeAndId(OrderType::Deposit, &order.id),
            )
            .await?;
        }
    }

    let withdraw_orders = bridge.filter_withdraw_event(&block_hash).await.unwrap();

    if withdraw_orders.is_empty() {
        //info!("Not found new filter_withdraw_event3 created at height {}", height);
    } else {
        info!(
            "filter_withdraw_event {:?} at height {}",
            withdraw_orders, height
        );
        for order in withdraw_orders {
            EthBridgeOrderEntity::update_single(
                BridgeOrderUpdater::Status(EthOrderStatus::Confirmed),
                BridgeOrderFilter::ByTypeAndId(OrderType::Withdraw, &order.id),
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn start() -> Result<()> {
    let mut last_process_height = get_last_process_height().await?;
    let bridge: EthContractClient<Bridge> = EthContractClient::<Bridge>::new()?;
    //let cli = EthContractClient::<crate::bridge_on_eth::Bridge>::new().await.unwrap();
    loop {
        let current_height = get_current_height().await?;
        info!(
            "current chain height1 {},wait for new block",
            current_height
        );

        if last_process_height == current_height {
            info!(
                "current chain height2 {},wait for new block",
                current_height
            );
            tokio::time::sleep(time::Duration::from_millis(1000)).await;
        } else if last_process_height < current_height {
            //初始化和监听最新区块复用了此逻辑(8区块的时候confirm，当前的pending)
            for height in last_process_height + 1..=current_height {
                listen_newest_block(&bridge, height).await?;
                listen_confirmed_block(&bridge, height - ETH_TX_CONFIRM_BLOCK_NUM as u64).await?;
            }
            last_process_height = current_height;
        } else {
            //it is impossible for big than current_height
            unreachable!()
        }
    }
}
