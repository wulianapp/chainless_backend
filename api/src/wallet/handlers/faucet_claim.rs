use crate::utils::{get_main_account, token_auth};
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, CoinType};
use common::error_code::BackendError;

use common::error_code::BackendRes;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FaucetClaimRequest {
    pub account_id: Option<String>,
}

pub async fn req(req: HttpRequest, req_data: FaucetClaimRequest) -> BackendRes<String> {
    let account = match req_data.account_id {
        Some(id) => id,
        None => {
            let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
            get_main_account(&user_id).await?
        }
    };

    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&account).await?;
    if master_list.len() != 1 {
        Err(BackendError::RequestParamInvalid("".to_string()))?;
    }
    let coin_list = get_support_coin_list();
    for coin in coin_list {
        let mut coin_cli: ContractClient<Coin> =
            ContractClient::<Coin>::new_update_cli(coin.clone()).await?;
        let amount = if coin.eq(&CoinType::ETH) {
            10000000000000000
        } else {
            100000000000000000000
        };
        let _balance = coin_cli.send_coin(&account, amount).await?;
    }
    Ok(None)
}
