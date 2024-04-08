/// listen tx status on chain
use common::{data_structures::wallet::TxStatusOnChain, *};
use models::{wallet_manage_record::{WalletManageRecordFilter, WalletManageRecordUpdater, WalletManageRecordView}, PsqlOp};
use tracing::debug;



#[tokio::main]
async fn main() {
    println!("Hello, world!");
    loop{
        //check manage_opcord
        let ops = WalletManageRecordView::find(
            WalletManageRecordFilter::ByStatus(
                &TxStatusOnChain::Pending
            )
        ).unwrap();

        for op in ops  {
            let tx_id = op.record.tx_ids.last().unwrap();
            debug!("start check tx {}",tx_id);
            let status = blockchain::general::tx_status(tx_id).await.unwrap();
            if status != TxStatusOnChain::Pending{
                let _ = WalletManageRecordView::update(
                    WalletManageRecordUpdater::Status(status), 
                    WalletManageRecordFilter::ByRecordId(&op.record.record_id)
                );
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            }

            //todo: try to call again
        }

        //todo: check transaction


        //todo: check bridge bind address
    }
}
