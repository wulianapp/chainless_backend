use common::data_structures::TxStatusOnChain;
use models::{
    wallet_manage_record::{
        WalletManageRecordEntity, WalletManageRecordFilter, WalletManageRecordUpdater,
    },
    PsqlOp,
};
use tracing::debug;

use anyhow::Result;

pub async fn start() -> Result<()> {
    loop {
        //check manage_opcord
        let ops = WalletManageRecordEntity::find(WalletManageRecordFilter::ByStatus(
            &TxStatusOnChain::Pending,
        ))
        .await?;

        for op in ops {
            //有些业务(如创建从设备换成主设备) 会产生多个txid，此时以最后一个id为准
            let tx_id = op.record.tx_ids.last().unwrap();
            debug!("start check tx {}", tx_id);
            let status = blockchain::general::tx_status(tx_id).await.unwrap();
            if status != TxStatusOnChain::Pending {
                let _ = WalletManageRecordEntity::update_single(
                    WalletManageRecordUpdater::Status(status),
                    WalletManageRecordFilter::ByRecordId(&op.record.record_id),
                )
                .await;
            }
            //todo: try to call again when failed
        }
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
