use crate::utils::token_auth;
use crate::wallet::CreateMainAccountRequest;
use actix_web::HttpRequest;
use blockchain::coin::Coin;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{get_support_coin_list, CoinType};
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::error_code::BackendRes;
use models::account_manager::{UserFilter, UserInfoView};
use models::PsqlOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use crate::wallet::BalanceListRequest;
use common::error_code::BackendError::ChainError;

#[derive(Deserialize, Serialize, Clone,Debug)]
pub struct AccountBalance {
    account_id: String,
    coin: CoinType,
    total_balance:u128,
    available_balance: u128,
    freezn_amount:u128,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum AccountType {
   Main,
   AllSub,
   Single(String),
   All,
}

pub async fn req(req: HttpRequest,request_data: BalanceListRequest) -> BackendRes<HashMap<String,Vec<AccountBalance>>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;

    let main_account = user_info.user_info.main_account;
    let coin_list = get_support_coin_list();
    let mul_cli = ContractClient::<MultiSig>::new()?;


    let check_accounts = match request_data.kind {
        AccountType::Main => vec![main_account],
        AccountType::AllSub => {
            let strategy = mul_cli
            .get_strategy(&main_account)
            .await?
            .ok_or(InternalError("11".to_string()))?;
            strategy.sub_confs
            .iter()
            .map(|x|x.0.to_string())
            .collect::<Vec<String>>()
        },
        AccountType::All => {
            let mut all = vec![main_account.clone()];
            let strategy = mul_cli
            .get_strategy(&main_account)
            .await?
            .ok_or(InternalError("11".to_string()))?;

            let mut sub = strategy.sub_confs
            .iter()
            .map(|x|x.0.to_string())
            .collect::<Vec<String>>();
            all.append(&mut sub);
            all
        },
        AccountType::Single(acc) => vec![acc],
    };


    let mut coin_balance_map = HashMap::new();
    for coin in &coin_list {
        let mut account_balance = vec![];
        for account in  check_accounts.iter().as_ref(){
            let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
            let balance_on_chain = if user_info.user_info.secruity_is_seted {
                coin_cli
                    .get_balance(&account)
                    .await?
                    .unwrap_or("0".to_string())
            } else {
                "0".to_string()
            };
            let freezn_amount = super::get_freezn_amount(&account, &coin);
            let total_balance = balance_on_chain.parse().unwrap();
            let available_balance = total_balance - freezn_amount;
            let balance = AccountBalance{
                account_id:account.clone(),
                coin: coin.clone(),
                total_balance,
                available_balance,
                freezn_amount,
            };
            account_balance.push(balance);
        }
        coin_balance_map.insert(coin.to_string(), account_balance);
    }

  
    Ok(Some(coin_balance_map))
}
