use crate::utils::token_auth;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::fees_call::FeesCall;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, CoinType};
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::error_code::{to_internal_error, BackendError};
use common::utils::math::coin_amount::raw2display;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::general::get_pg_pool_connect;
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use tracing::debug;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SingleBalanceRequest {
    coin: String,
    account_id: Option<String>,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SingleBalanceResponse {
    total_balance: String,
    total_dollar_value: String,
    total_rmb_value: String,
    hold_limit: Option<String>,
}

pub async fn req(
    req: HttpRequest,
    request_data: SingleBalanceRequest,
) -> BackendRes<SingleBalanceResponse> {

    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    let user_info = UserInfoEntity::find_single(UserFilter::ById(&user_id))
        .await?
        .into_inner();
    let main_account = user_info.main_account.clone().unwrap();

    let SingleBalanceRequest { coin, account_id } = request_data;
    let coin: CoinType = coin
        .parse()
        .map_err(|_e| BackendError::RequestParamInvalid("coin not support".to_string()))?;
    let multi_cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let (dist_account, hold_limit) = match account_id {
        Some(account) => {
            let strategy = multi_cli
                .get_strategy(&main_account)
                .await?
                .ok_or("not regist main_account")?;

            let sub_conf = strategy.sub_confs.get(account.as_str()).ok_or(
                BackendError::RequestParamInvalid("account is not subaccount".to_string()),
            )?;
            let hold_limit = raw2display(sub_conf.hold_value_limit);
            (account, Some(hold_limit))
        }
        None => (main_account.clone(), None),
    };

    let coin_cli: ContractClient<Coin> =
        ContractClient::<Coin>::new_query_cli(coin.clone()).await?;
    let balance = coin_cli
        .get_balance(&dist_account)
        .await?
        .unwrap_or("0".to_string());

    let total_balance = balance.parse().map_err(to_internal_error)?;
    debug!("coin:{},total_balance:{}", coin, total_balance);
    let total_dollar_value = super::get_value(&coin, total_balance).await;
    let total_rmb_value = total_dollar_value / 7;
    let balance = SingleBalanceResponse {
        total_balance: raw2display(total_balance),
        total_dollar_value: raw2display(total_dollar_value),
        total_rmb_value: raw2display(total_rmb_value),
        hold_limit,
    };
    Ok(Some(balance))
}
