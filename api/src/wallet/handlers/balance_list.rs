use crate::utils::token_auth;
use crate::wallet::{BalanceDetail, CreateMainAccountRequest};
use crate::wallet::{BalanceListRequest, BalanceListResponse};
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::fees_call::FeesCall;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, CoinType};
use common::error_code::BackendError::ChainError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::error_code::{parse_str, BackendError};
use common::utils::math::coin_amount::raw2display;
use models::account_manager::{UserFilter, UserInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use tracing::debug;

#[derive(Deserialize, Serialize, Clone)]
pub enum AccountType {
    Main,
    AllSub,
    Single(String),
    All,
}

pub async fn req(
    req: HttpRequest,
    request_data: BalanceListRequest,
) -> BackendRes<BalanceListResponse> {
    let user_id = token_auth::validate_credentials(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;

    let main_account = user_info.user_info.main_account;
    let coin_list = get_support_coin_list();
    let mul_cli = ContractClient::<MultiSig>::new().await?;

    let check_accounts = match request_data.kind {
        AccountType::Main => vec![main_account.clone()],
        AccountType::AllSub => {
            let strategy = mul_cli
                .get_strategy(&main_account)
                .await?
                .ok_or(InternalError("11".to_string()))?;
            strategy
                .sub_confs
                .iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
        }
        AccountType::All => {
            let mut all = vec![main_account.clone()];
            if main_account.ne("") {
                let strategy = mul_cli
                    .get_strategy(&main_account)
                    .await?
                    .ok_or(InternalError("11".to_string()))?;

                let mut sub = strategy
                    .sub_confs
                    .iter()
                    .map(|x| x.0.to_string())
                    .collect::<Vec<String>>();
                all.append(&mut sub);
            }
            all
        }
        AccountType::Single(acc) => vec![acc],
    };

    let multi_cli = blockchain::ContractClient::<MultiSig>::new().await?;
    let mut coin_balance_map = vec![];
    for coin in coin_list {
        let mut account_balance = vec![];

        for (index, account) in check_accounts.iter().enumerate() {
            let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new_with_type(coin.clone()).await?;
            let (balance_on_chain, hold_limit) = if user_info.user_info.secruity_is_seted {
                let balance = coin_cli
                    .get_balance(account)
                    .await?
                    .unwrap_or("0".to_string());
                let hold_limit = if index == 0 {
                    None
                } else {
                    let strategy = multi_cli.get_strategy(&main_account).await?.ok_or("")?;
                    let sub_confs = strategy.sub_confs;
                    let hold_limit = sub_confs.get(account.as_str()).ok_or("")?.hold_value_limit;
                    Some(raw2display(hold_limit))
                };
                (balance, hold_limit)
            } else {
                ("0".to_string(), Some("0.0".to_string()))
            };
            let freezn_amount = super::get_freezn_amount(account, &coin);
            let total_balance = parse_str(balance_on_chain)?;
            debug!(
                "coin:{},total_balance:{},freezn_amount:{}",
                coin, total_balance, freezn_amount
            );
            let available_balance = total_balance - freezn_amount;
            let total_dollar_value = super::get_value(&coin, total_balance).await;
            let total_rmb_value = total_dollar_value / 7;
            let balance = BalanceDetail {
                account_id: account.clone(),
                coin: coin.clone(),
                total_balance: raw2display(total_balance),
                available_balance: raw2display(available_balance),
                freezn_amount: raw2display(freezn_amount),
                total_dollar_value: raw2display(total_dollar_value),
                total_rmb_value: raw2display(total_rmb_value),
                hold_limit,
            };
            account_balance.push(balance);
        }
        coin_balance_map.push((coin, account_balance));
    }

    Ok(Some(coin_balance_map))
}
