/// listen tx status on chain
pub mod task;
use common::{data_structures::TxStatusOnChain, *};
use models::{
    wallet_manage_record::{
        WalletManageRecordFilter, WalletManageRecordUpdater, WalletManageRecordView,
    },
    PsqlOp,
};
use tracing::debug;
use clap::Parser;
use tracing::info;
use anyhow::Result;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    task: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main()  -> Result<()>{
    common::log::init_logger();
    let args = Args::parse();
    /*** 
    loop {
        //check manage_opcord
        let ops = WalletManageRecordView::find(WalletManageRecordFilter::ByStatus(
            &TxStatusOnChain::Pending,
        ))
        .unwrap();

        for op in ops {
            //todo: 目前的txid是bs58的待修复
            let tx_id = op.record.tx_ids.last().unwrap();
            debug!("start check tx {}", tx_id);
            let status = blockchain::general::tx_status(tx_id).await.unwrap();
            if status != TxStatusOnChain::Pending {
                let _ = WalletManageRecordView::update_single(
                    WalletManageRecordUpdater::Status(status),
                    WalletManageRecordFilter::ByRecordId(&op.record.record_id),
                );
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            }

            //todo: try to call again
        }

        //todo: check transaction

        //todo: check bridge bind address
    }*/
    match args.task.as_str() {
        "eth_bridge" => {
            info!("start task listening on eth_bridge");
            let _res = task::eth_bridge::start().await?;
            
        },
        "chainless_relayer" => {
            info!("start task listening on chainless_relayer");

        },
        "chainless_user" => {
            info!("start task listening on chainless_user");
        },
        _ => panic!("unknown task"),
    }
    Ok(())

}
