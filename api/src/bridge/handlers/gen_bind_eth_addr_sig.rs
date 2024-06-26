use actix_web::HttpRequest;
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::KeyRole;

use serde::{Deserialize, Serialize};
//use log::debug;

use crate::utils::{get_user_context, token_auth};
use crate::wallet::handlers::*;

use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenBindEthAddrSigRequest {
    eth_addr: String,
}

pub async fn req(req: HttpRequest, request_data: GenBindEthAddrSigRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    let GenBindEthAddrSigRequest { eth_addr } = request_data.clone();

    let bridge_cli = ContractClient::<Bridge>::new_query_cli().await?;

    let sig = bridge_cli.sign_bind_info(&main_account, &eth_addr).await?;
    println!("sig {} ", sig);

    Ok(Some(sig))
}
