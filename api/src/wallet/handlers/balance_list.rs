use crate::utils::token_auth;
use actix_web::HttpRequest;
use blockchain::coin::Coin;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::{get_support_coin_list, CoinType};

use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use common::error_code::{parse_str};
use common::utils::math::coin_amount::raw2display;
use models::account_manager::{UserFilter, UserInfoEntity};

use models::PsqlOp;
use serde::{Deserialize, Serialize};



use tracing::debug;

#[derive(Deserialize, Serialize, Clone)]
pub enum AccountType {
    Main,
    AllSub,
    Single(String),
    All,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceListRequest {
    kind: AccountType,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BalanceDetail {
    account_id: String,
    coin: CoinType,
    total_balance: String,
    total_dollar_value: String,
    total_rmb_value: String,
    available_balance: String,
    freezn_amount: String,
    hold_limit: Option<String>,
}
pub type BalanceListResponse = Vec<(CoinType, Vec<BalanceDetail>)>;

pub async fn req(
    req: HttpRequest,
    request_data: BalanceListRequest,
) -> BackendRes<BalanceListResponse> {
    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    let user_info = UserInfoEntity::find_single(UserFilter::ById(&user_id))
        .await?
        .into_inner();

    let main_account = user_info.main_account.clone();
    let coin_list = get_support_coin_list();
    let mul_cli = ContractClient::<MultiSig>::new_query_cli().await?;

    let check_accounts = match request_data.kind {
        AccountType::Main => vec![main_account.clone()],
        AccountType::AllSub => {
            if main_account.is_some() {
                let strategy = mul_cli
                    .get_strategy(&main_account.unwrap())
                    .await?
                    .ok_or(InternalError("".to_string()))?;
                strategy
                    .sub_confs
                    .iter()
                    .map(|x| Some(x.0.to_string()))
                    .collect::<Vec<Option<String>>>()
            } else {
                vec![]
            }
        }
        AccountType::All => {
            let mut all = vec![main_account.clone()];
            if main_account.is_some() {
                let strategy = mul_cli
                    .get_strategy(&main_account.unwrap())
                    .await?
                    .ok_or(InternalError("".to_string()))?;

                let mut sub = strategy
                    .sub_confs
                    .iter()
                    .map(|x| Some(x.0.to_string()))
                    .collect::<Vec<Option<String>>>();
                all.append(&mut sub);
            }
            all
        }
        AccountType::Single(acc) => vec![Some(acc)],
    };

    let multi_cli = blockchain::ContractClient::<MultiSig>::new_query_cli().await?;
    let mut coin_balance_map = vec![];
    for coin in coin_list {
        let mut account_balance = vec![];

        for (index, account) in check_accounts.iter().enumerate() {
            let coin_cli: ContractClient<Coin> =
                ContractClient::<Coin>::new_query_cli(coin.clone()).await?;
            let (balance_on_chain, hold_limit) = if user_info.main_account.is_some() {
                let balance = coin_cli
                    .get_balance(account.as_ref().unwrap())
                    .await?
                    .unwrap_or("0".to_string());

                let hold_limit = if index == 0 {
                    None
                } else {
                    let strategy = multi_cli
                        .get_strategy(user_info.main_account.as_ref().unwrap())
                        .await?
                        .ok_or("")?;
                    let sub_confs = strategy.sub_confs;
                    let hold_limit = sub_confs
                        .get(account.as_ref().unwrap())
                        .ok_or("")?
                        .hold_value_limit;
                    Some(raw2display(hold_limit))
                };

                (balance, hold_limit)
            } else {
                ("0".to_string(), Some("0.0".to_string()))
            };
            let freezn_amount = if account.is_none() {
                0
            } else {
                super::get_freezn_amount(account.as_ref().unwrap(), &coin).await
            };
            let total_balance = parse_str(balance_on_chain)?;
            debug!(
                "coin:{},total_balance:{},freezn_amount:{}",
                coin, total_balance, freezn_amount
            );
            let available_balance = total_balance - freezn_amount;
            let total_dollar_value = super::get_value(&coin, total_balance).await;
            let total_rmb_value = total_dollar_value / 7;
            let balance = BalanceDetail {
                account_id: account.clone().unwrap_or("".to_string()),
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
