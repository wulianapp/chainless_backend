use actix_web::HttpRequest;
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;

use crate::utils::{get_main_account, token_auth};

use common::error_code::BackendRes;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, _, _device_id, _) = token_auth::validate_credentials(&req).await?;
    let main_account = get_main_account(&user_id).await?;
    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;
    let eth_addr = bridge_cli.get_binded_eth_addr(&main_account).await?;
    Ok(eth_addr)
}
