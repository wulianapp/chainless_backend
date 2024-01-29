extern crate rustc_serialize;

use postgres::Row;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};
use common::data_structures::secret_store::SecretStore;

use crate::vec_str2array_text;
use common::data_structures::wallet::Wallet;

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretView {
    pub secret_store: SecretStore,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub enum SecretFilter {
    ByAccountId(u32),
}

impl SecretFilter {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            Self::ByAccountId(id) => {
                format!("account_id={} ", id)
            }
        };
        filter_str
    }
}

pub fn get_secret(filter: SecretFilter) -> Vec<SecretView> {
    let sql = format!(
        "select account_id,\
         user_id,\
         master_encrypted_prikey,\
         servant_encrypted_prikeys,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from secret_store where {}",
        filter.to_string()
    );
    let execute_res = crate::query(sql.as_str()).unwrap();
    debug!("get_snapshot: raw sql {}", sql);
    let gen_view = |row: &Row| SecretView {
        secret_store: SecretStore {
            account_id: row.get(0),
            user_id: row.get::<usize, i32>(1) as u32,
            master_encrypted_prikey: row.get(2),
            servant_encrypted_prikeys: row.get::<usize, Vec<String>>(3),
        },
        updated_at: row.get(4),
        created_at: row.get(5),
    };
    execute_res
        .iter()
        .map(|x| gen_view(x))
        .collect::<Vec<SecretView>>()
}

pub fn single_insert(data: &SecretStore) -> Result<(), String> {
    let SecretStore {
        account_id,
        user_id,
        master_encrypted_prikey,
        servant_encrypted_prikeys,
    } = data;

    let servant_keys_str = vec_str2array_text(servant_encrypted_prikeys.to_owned());

    let sql = format!(
        "insert into secret_store (\
         account_id,\
         user_id,\
         master_encrypted_prikey,\
         servant_encrypted_prikeys \
         ) values ('{}',{},{},{});",
        account_id,
        user_id,
        master_encrypted_prikey,
        servant_keys_str
    );
    println!("row sql {} rows", sql);

    let execute_res = crate::execute(sql.as_str()).map_err(|x| x.to_string())?;
    info!("success insert {} rows", execute_res);

    Ok(())
}
