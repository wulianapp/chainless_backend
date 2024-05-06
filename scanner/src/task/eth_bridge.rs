
use std::time;

use blockchain::bridge_on_near::{BridgeOrder};
use common::data_structures::bridge::{self, *};
use common::data_structures::bridge::EthBridgeOrder;
use models::eth_bridge_order::{BridgeOrderFilter, EthBridgeOrderView};
use models::PsqlOp;
use blockchain::eth_cli::general::*;
use anyhow::Result;
use blockchain::bridge_on_eth::{self, Bridge};
use blockchain::eth_cli::EthContractClient;
use tracing::info;


const CONFIRM_HEIGHT: u64 = 2;

//如果没历史监控数据，则从固定检查点开始扫,如果有则从历史数据中的最后高度开始扫
pub async fn get_last_process_height() -> Result<u64> {
    let last_order = EthBridgeOrderView::find(BridgeOrderFilter::Limit(1))?;
    if last_order.is_empty(){
        //Ok(get_current_block().await)
        Ok(1270960)
    }else {
        Ok(last_order[0].order.height)
    }
}


pub async fn start() -> Result<()>{
    let mut last_process_height = get_last_process_height().await?;
    let bridge = EthContractClient::<Bridge>::new()?;
    //let cli = EthContractClient::<crate::bridge_on_eth::Bridge>::new().unwrap();
    loop {
        let current_height = get_current_height().await?;
        info!("current chain height1 {},wait for new block", current_height);
        let current_confirmed_height = current_height - CONFIRM_HEIGHT;

        //todo: 8区块的时候confirm，之前pending
        //it is impossible for big than current_confirmed_height
        if last_process_height >= current_confirmed_height{
            info!("current chain height2 {},wait for new block", current_height);
            tokio::time::sleep(time::Duration::from_millis(1000)).await;
        } else {
            //规避RPC阻塞等网络问题导致的没有及时获取到最新块高，以及系统重启时期对离线期间区块的处理
            for height in last_process_height + 1..=current_confirmed_height
            {
                //tokio::time::sleep(time::Duration::from_millis(1000)).await;
                //info!("check height {}", height);
                let block_hash = get_block(height)
                    .await?
                    .unwrap()
                    .hash
                    .unwrap();
                let block_hash = hex::encode(block_hash.as_bytes());
                //info!("check block_hash {}", block_hash);

                let deposit_orders = bridge
                    .filter_deposit_event(&block_hash)
                    .await
                    .unwrap();
                if deposit_orders.is_empty() {
                    //info!("Not found deposit0 orders created at height {}",height);
                } else {
                    info!("filter_deposit_event {:?} at height {}", deposit_orders, height);
                    //todo: batch insert
                    for order in deposit_orders {
                       let order =  EthBridgeOrderView::new_with_specified(
                        &order.id, 
                        &order.chainless_acc,
                        &order.eth_addr,
                        order.order_type,
                        order.coin,
                        order.amount,
                        "Confirmed",
                        height
                        );
                        order.insert()?;
                    }
                    tokio::time::sleep(time::Duration::from_millis(1000)).await;
                }

                let withdraw_orders = bridge
                    .filter_withdraw_event(&block_hash)
                    .await
                    .unwrap();

                if withdraw_orders.is_empty() {
                    //info!("Not found new filter_withdraw_event3 created at height {}", height);
                } else {
                    info!("filter_withdraw_event {:?} at height {}", withdraw_orders, height);
                    for order in withdraw_orders {
                        let order =  EthBridgeOrderView::new_with_specified(
                         &order.id, 
                         &order.chainless_acc,
                         &order.eth_addr,
                         order.order_type,
                         order.coin,
                         order.amount,
                         "Confirmed",
                         height
                         );
                         order.insert()?;
                     }
                    tokio::time::sleep(time::Duration::from_millis(1000)).await;    
                }
            }
            last_process_height = current_confirmed_height;
        }
    }
}