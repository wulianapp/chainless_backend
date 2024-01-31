extern crate rustc_serialize;

use postgres::Row;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};

use crate::vec_str2array_text;
use common::data_structures::wallet::Wallet;
use common::error_code::{BackendError};
use common::http::BackendRes;

#[derive(Deserialize, Serialize, Debug)]
pub struct WalletView {
    pub wallet: Wallet,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub enum WalletFilter {
    ByUserId(u32),
}

impl WalletFilter {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            WalletFilter::ByUserId(uid) => {
                format!("user_id={} ", uid)
            }
        };
        filter_str
    }
}

pub fn get_wallet(filter: WalletFilter) -> Result<Vec<WalletView>,BackendError> {
    let sql = format!(
        "select user_id,\
         account_id,\
         sub_pubkeys,\
         sign_strategies,\
         participate_device_ids,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from wallet where {}",
        filter.to_string()
    );
    let execute_res = crate::query(sql.as_str())?;
    info!("get_snapshot: raw sql {}", sql);
    if execute_res.len() > 1 {
        //todo:throw error
        panic!("_tmp");
    }
    let gen_view = |row: &Row| WalletView {
        wallet: Wallet {
            user_id: row.get::<usize, i32>(0) as u32,
            account_id: row.get(1),
            sub_pubkeys: row.get::<usize, Vec<String>>(2),
            sign_strategies: row.get::<usize, Vec<String>>(3),
            participate_device_ids: row.get::<usize, Vec<String>>(4),
        },
        updated_at: row.get(5),
        created_at: row.get(6),
    };
    Ok(execute_res
        .iter()
        .map(|x| gen_view(x))
        .collect::<Vec<WalletView>>())
}

pub fn single_insert(data: &Wallet) -> Result<(), BackendError> {
    let Wallet {
        user_id,
        account_id,
        sub_pubkeys,
        sign_strategies,
        participate_device_ids,
    } = data;

    let sql = format!(
        "insert into wallet (\
         user_id,\
         account_id,\
         sub_pubkeys,\
         sign_strategies,\
         participate_device_ids\
         ) values ({},'{}',{},{},{});",
        user_id,
        account_id,
        vec_str2array_text(sub_pubkeys.to_owned()),
        vec_str2array_text(sign_strategies.to_owned()),
        vec_str2array_text(participate_device_ids.to_owned())
    );
    println!("row sql {} rows", sql);

    let execute_res = crate::execute(sql.as_str())?;
    info!("success insert {} rows", execute_res);

    Ok(())
}

pub fn update(
    sub_pubkeys: Vec<String>,
    sign_strategies: Vec<String>,
    participate_device_ids: Vec<String>,
    filter: WalletFilter,
) -> Result<(),BackendError>{
    let sql = format!(
        "update wallet set (sub_pubkeys,sign_strategies,participate_device_ids)=\
         ({},{},{}) where {}",
        vec_str2array_text(sub_pubkeys),
        vec_str2array_text(sign_strategies),
        vec_str2array_text(participate_device_ids),
        filter.to_string()
    );
    info!("start update wallet {} ", sql);
    let execute_res = crate::execute(sql.as_str())?;
    info!("success update trade {} rows", execute_res);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_braced_models_wallet() {
        env::set_var("SERVICE_MODE", "test");
        crate::general::table_all_clear();

        let wallet = Wallet {
            user_id: 1,
            account_id: "00000000000000001".to_string(),
            sub_pubkeys: vec!["012".to_string(), "345".to_string()],
            sign_strategies: vec!["012".to_string(), "345".to_string()],
            participate_device_ids: vec!["012".to_string(), "345".to_string()],
        };

        println!("start insert");
        single_insert(&wallet).unwrap();
        println!("start query");
        let res = get_wallet(WalletFilter::ByUserId(1));
        println!("select_res {},", serde_json::to_string(&res).unwrap());
        let _res = update(vec![], vec![], vec![], WalletFilter::ByUserId(1));
        let res = get_wallet(WalletFilter::ByUserId(1));
        println!("select_res {},", serde_json::to_string(&res).unwrap());
    }
}
