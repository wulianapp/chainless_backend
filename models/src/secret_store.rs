extern crate rustc_serialize;

use postgres::Row;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};
use common::data_structures::secret_store::SecretStore;

use crate::vec_str2array_text;
use common::data_structures::wallet::Wallet;
use common::error_code::{BackendError};

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretView {
    pub secret_store: SecretStore,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub enum SecretFilter<'a> {
    ByAccountId(&'a str),
}

impl SecretFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            Self::ByAccountId(id) => {
                format!("account_id='{}' ", id)
            }
        };
        filter_str
    }
}

pub fn get_secret(filter: SecretFilter) -> Result<Vec<SecretView>,BackendError> {
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
    let execute_res = crate::query(sql.as_str())?;
    debug!("get_secret: raw sql {}", sql);
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
    Ok(
        execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<SecretView>>()
    )
}

pub fn single_insert(data: &SecretStore) -> Result<(), BackendError> {
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
         ) values ('{}',{},'{}',{});",
        account_id,
        user_id,
        master_encrypted_prikey,
        servant_keys_str
    );
    println!("row sql {} rows", sql);

    let execute_res = crate::execute(sql.as_str())?;
    info!("success insert {} rows", execute_res);

    Ok(())
}

pub fn update_servant(new_servants: Vec<String>, filter: SecretFilter) -> Result<(),BackendError>{
    let new_servant_str = super::vec_str2array_text(new_servants);
    let sql = format!(
        "update secret_store set servant_encrypted_prikeys={} where {}",
        new_servant_str,
        filter.to_string()
    );
    debug!("start update orders {} ", sql);
    let execute_res = crate::execute(sql.as_str())?;
    debug!("success update orders {} rows", execute_res);
    Ok(())
}
