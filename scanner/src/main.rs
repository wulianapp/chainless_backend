/// listen tx status on chain
pub mod task;

use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    task: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    common::log::init_logger();
    let args = Args::parse();
    //剥离wallet_manage和coin_transfer的原因是考虑到是否relayer签名、是否重试、以及业务的解耦
    match args.task.as_str() {
        "eth_bridge" => {
            info!("start task listening on eth_bridge");
            let _res = task::eth_bridge::start().await?;
        }
        "chainless_wallet_manage" => {
            info!("start task listening on chainless_wallet_manage");
            let _res = task::chainless_wallet_manage::start().await?;
        }
        "chainless_coin_transfer" => {
            info!("start task listening on chainless_coin_transfer");
            let _res = task::chainless_coin_transfer::start().await?;
        }
        "refund_fee" => {
            //todo:
            info!("start task refund_fee");
        }
        _ => panic!("unknown task"),
    }
    Ok(())
}
