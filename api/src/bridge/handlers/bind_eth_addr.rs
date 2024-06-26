use actix_web::HttpRequest;
use blockchain::bridge_on_near::Bridge;
use blockchain::ContractClient;
use common::data_structures::KeyRole;

use serde::{Deserialize, Serialize};
//use log::debug;
use tracing::debug;

use crate::utils::{get_user_context, token_auth};
use crate::wallet::handlers::*;

use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BindEthAddrRequest {
    eth_addr: String,
    user_eth_sig: String,
}

pub async fn req(req: HttpRequest, request_data: BindEthAddrRequest) -> BackendRes<String> {
    //todo: check jwt token
    debug!("start reset_password");

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    let BindEthAddrRequest {
        eth_addr,
        user_eth_sig,
    } = request_data.clone();

    let mut bridge_cli = ContractClient::<Bridge>::new_update_cli().await?;

    //todo: 应该校验用户的签名而不是自己的
    /***
    if !bridge_cli.verify_eth_bind_sign(&eth_addr,&main_account,&user_eth_sig){
        Err(BackendError::InternalError("".to_string()))?;
    }
    **/

    let bind_res = bridge_cli
        .bind_eth_addr(&main_account, &eth_addr, &user_eth_sig)
        .await?;
    println!("bind_res {} ", bind_res);

    Ok(None)
}
