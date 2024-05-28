use common::data_structures::TxStatusOnChain;
use models::{
    general::get_pg_pool_connect,
    wallet_manage_record::{
        WalletManageRecordEntity, WalletManageRecordFilter, WalletManageRecordUpdater,
    },
    PsqlOp,
};
use tracing::debug;

use anyhow::Result;

pub async fn start() -> Result<()> {
    let mut db_cli = get_pg_pool_connect().await?;

    loop {
        //check manage_opcord
        let ops = WalletManageRecordEntity::find(
            WalletManageRecordFilter::ByStatus(&TxStatusOnChain::Pending),
            &mut db_cli,
        )
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
                    &mut db_cli,
                )
                .await;
            }
            //todo: try to call again when relayer operate
        }
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }
}
