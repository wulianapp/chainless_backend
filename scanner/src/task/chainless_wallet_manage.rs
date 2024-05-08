use common::data_structures::TxStatusOnChain;
use models::{
    wallet_manage_record::{
        WalletManageRecordFilter, WalletManageRecordUpdater, WalletManageRecordView,
    },
    PsqlOp,
};
use tracing::debug;

use anyhow::Result;

pub async fn start() -> Result<()> {
    loop {
        //check manage_opcord
        let ops = WalletManageRecordView::find(WalletManageRecordFilter::ByStatus(
            &TxStatusOnChain::Pending,
        ))?;

        for op in ops {
            //有些业务(如创建从设备换成主设备) 会产生多个txid，此时以最后一个id为准
            let tx_id = op.record.tx_ids.last().unwrap();
            debug!("start check tx {}", tx_id);
            let status = blockchain::general::tx_status(tx_id).await.unwrap();
            if status != TxStatusOnChain::Pending {
                let _ = WalletManageRecordView::update_single(
                    WalletManageRecordUpdater::Status(status),
                    WalletManageRecordFilter::ByRecordId(&op.record.record_id),
                );
            }
            //todo: try to call again when relayer operate
        }
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
