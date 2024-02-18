extern crate rustc_serialize;

use std::fmt;
use std::fmt::Display;
use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::secret_store::SecretStore;
use serde::{Deserialize, Serialize};
use common::data_structures::wallet::{CoinTxStatus, StrategyMessageType};

use crate::{PsqlOp, vec_str2array_text};

use common::error_code::BackendError;


#[derive(Clone, Debug)]
pub enum SecretUpdater {
    Servant(Vec<String>),
}

impl fmt::Display for SecretUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretUpdater::Servant(keys) =>  {
                let new_servant_str = super::vec_str2array_text(keys.to_owned());
                format!("servant_encrypted_prikeys={} ", new_servant_str)
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum SecretFilter {
    ByAccountId(String),
}

impl fmt::Display for SecretFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretFilter::ByAccountId(id) =>  format!("account_id='{}' ", id),
        };
        write!(f, "{}", description)
    }
}



pub struct SecretStore2{
    pub account_id: String,
    pub user_id: u32,
    pub master_encrypted_prikey: String,
    pub servant_encrypted_prikeys: Vec<String>,
}

impl SecretStore2{
    pub fn new_with_specified(account_id:String,
                              user_id:u32,
                              master_encrypted_prikey:String
    ) -> Self{
        SecretStore2{
            account_id,
            user_id,
            master_encrypted_prikey,
            servant_encrypted_prikeys: vec![]
        }
    }
}

impl PsqlOp for SecretStore2{

    type UpdateContent = SecretUpdater;
    type FilterContent = SecretFilter;

    fn find(filter: SecretFilter) -> Result<Vec<SecretStore2>, BackendError> {
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
        let gen_view = |row: &Row|{
            SecretStore2 {
                account_id: row.get(0),
                user_id: row.get::<usize, i32>(1) as u32,
                master_encrypted_prikey: row.get(2),
                servant_encrypted_prikeys: row.get::<usize, Vec<String>>(3),
            }
        };

        Ok(execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<SecretStore2>>())
    }
    fn update(new_value: SecretUpdater, filter: SecretFilter) -> Result<(), BackendError> {
        let sql = format!(
            "update secret_store set {} where {}",
            new_value.to_string(),
            filter.to_string()
        );
        debug!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        debug!("success update orders {} rows", execute_res);
        Ok(())
    }

    fn insert(&self) -> Result<(), BackendError> {
        let SecretStore2 {
            account_id,
            user_id,
            master_encrypted_prikey,
            servant_encrypted_prikeys,
        } = self;

        let servant_keys_str = vec_str2array_text(servant_encrypted_prikeys.to_owned());

        let sql = format!(
            "insert into secret_store (\
         account_id,\
         user_id,\
         master_encrypted_prikey,\
         servant_encrypted_prikeys \
         ) values ('{}',{},'{}',{});",
            account_id, user_id, master_encrypted_prikey, servant_keys_str
        );
        println!("row sql {} rows", sql);

        let execute_res = crate::execute(sql.as_str())?;
        info!("success insert {} rows", execute_res);

        Ok(())
    }

}



#[derive(Deserialize, Serialize, Debug)]
pub struct SecretView {
    pub secret_store: SecretStore,
    pub updated_at: String,
    pub created_at: String,
}