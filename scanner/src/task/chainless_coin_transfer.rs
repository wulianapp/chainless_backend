use common::data_structures::TxStatusOnChain;
use models::coin_transfer::CoinTxView;
use models::{
    coin_transfer::{CoinTxFilter, CoinTxUpdater},
    PsqlOp,
};
use tracing::{debug, error};

use anyhow::Result;

pub async fn start() -> Result<()> {
    loop {
        //check manage_opcord
        let txs = CoinTxView::find(CoinTxFilter::ByChainStatus(TxStatusOnChain::Pending))?;

        for tx in txs {
            let tx_id = if let Some(txid) = tx.transaction.tx_id {
                txid
            } else {
                error!("pending tx have no txid?");
                continue;
            };

            debug!("start check tx {}", tx_id);
            let status = blockchain::general::tx_status(&tx_id).await?;
            if status != TxStatusOnChain::Pending {
                CoinTxView::update_single(
                    CoinTxUpdater::StageChainStatus(tx.transaction.stage, status),
                    CoinTxFilter::ByOrderId(&tx.transaction.order_id),
                )?;
            }
            //todo: try to call again,if main2sub or sub2main
        }
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
