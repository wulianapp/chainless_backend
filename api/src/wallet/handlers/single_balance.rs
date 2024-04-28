use crate::utils::token_auth;
use crate::wallet::{BalanceDetail, CreateMainAccountRequest};
use crate::wallet::{SingleBalanceRequest, SingleBalanceResponse};
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::fees_call::FeesCall;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, CoinType};
use common::error_code::BackendError;
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::utils::math::coin_amount::raw2display;
use models::account_manager::{UserFilter, UserInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use tracing::debug;

pub async fn req(
    req: HttpRequest,
    request_data: SingleBalanceRequest,
) -> BackendRes<SingleBalanceResponse> {
    let user_id = token_auth::validate_credentials(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;

    let SingleBalanceRequest { coin, account_id } = request_data;
    let coin: CoinType = coin.parse().map_err(|e| BackendError::RequestParamInvalid("coin not support".to_string()))?;
    let multi_cli = ContractClient::<MultiSig>::new()?;
    let (dist_account,hold_limit) = match account_id {
        Some(account) => {
            let strategy = multi_cli.get_strategy(&main_account).await?;
            let sub_confs = strategy.unwrap().sub_confs;
            let sub_conf = sub_confs
            .get(account.as_str())
            .ok_or(BackendError::RequestParamInvalid("account is not subaccount".to_string()))?;
            let hold_limit = raw2display(sub_conf.hold_value_limit);
            (account,Some(hold_limit))
        },
        None => (main_account.clone(),None)
    };

    let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
    let balance = coin_cli
    .get_balance(&dist_account)
    .await?
    .unwrap_or("0".to_string());

    let total_balance = balance.parse().unwrap();
    debug!("coin:{},total_balance:{}",coin, total_balance);
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