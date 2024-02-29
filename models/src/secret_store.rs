extern crate rustc_serialize;

use std::fmt;
use std::fmt::Display;
use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use common::data_structures::SecretKeyState;
use serde::{Deserialize, Serialize};
use common::data_structures::wallet::{CoinTxStatus, StrategyMessageType};
use slog_term::PlainSyncRecordDecorator;

use crate::{PsqlOp, vec_str2array_text};

use common::error_code::BackendError;


#[derive(Debug)]
pub enum SecretUpdater {
    EncrypedPrikey((String,String)),
    State(SecretKeyState),
}

impl fmt::Display for SecretUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretUpdater::EncrypedPrikey((by_password,by_answer)) =>  {
                format!("encrypted_prikey_by_password={},
                encrypted_prikey_by_answer={}", by_password,by_answer)
            },
            SecretUpdater::State(new_state) =>  {
                format!("state='{}'", new_state.to_string())
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum SecretFilter {
    ByPubkey(String),
    BySittingPubkey(String),
}

impl fmt::Display for SecretFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretFilter::ByPubkey(key) =>  format!("pubkey='{}' ", key),
            SecretFilter::BySittingPubkey(key) =>  format!("state='Sitting' and pubkey='{}' ", key),
        };
        write!(f, "{}", description)
    }
}



#[derive(Deserialize, Serialize, Debug,PartialEq)]
pub struct SecretStoreView{
    pub secret_store: SecretStore,
    pub updated_at: String,
    pub created_at: String,
}

impl SecretStoreView{
    pub fn new_with_specified(pubkey:&str,
                              user_id:u32,
                              encrypted_prikey_by_password:&str,
                              encrypted_prikey_by_answer:&str

    ) -> Self{
        SecretStoreView{
            secret_store:    SecretStore{
                pubkey:pubkey.to_string(),
                state: SecretKeyState::Sitting,
                user_id,
                encrypted_prikey_by_password: encrypted_prikey_by_password.to_string(),
                encrypted_prikey_by_answer: encrypted_prikey_by_answer.to_string(),
            },
            updated_at: "".to_string(),
            created_at: "".to_string()
        }
    }
}

impl PsqlOp for SecretStoreView{

    type UpdateContent = SecretUpdater;
    type FilterContent = SecretFilter;

    fn find(filter: SecretFilter) -> Result<Vec<SecretStoreView>, BackendError> {
        let sql = format!(
            "select 
            pubkey,\
            state,\
            user_id,\
            encrypted_prikey_by_password,\
            encrypted_prikey_by_answer,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from secret_store where {}",
            filter.to_string()
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row|{
            SecretStoreView {
                secret_store: SecretStore{
                    pubkey: row.get(0),
                    state: row.get::<usize, String>(1).parse().unwrap(),
                    user_id: row.get::<usize, i32>(2) as u32,
                    encrypted_prikey_by_password: row.get(3),
                    encrypted_prikey_by_answer: row.get(4),
                },
                updated_at: row.get(4),
                created_at: row.get(5),
            }
        };

        Ok(execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<SecretStoreView>>())
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
        let SecretStore {
            pubkey,
            state,
            user_id,
            encrypted_prikey_by_password,
            encrypted_prikey_by_answer,
        } = &self.secret_store;

        let sql = format!(
            "insert into secret_store (\
                pubkey,\
                state,\
                user_id,\
                encrypted_prikey_by_password,\
                encrypted_prikey_by_answer\
         ) values ('{}','{}',{},'{}','{}');",
         pubkey,state.to_string(),user_id,
         encrypted_prikey_by_password, encrypted_prikey_by_answer
        );
        debug!("row sql {} rows", sql);
        let execute_res = crate::execute(sql.as_str())?;
        Ok(())
    }

}



#[derive(Deserialize, Serialize, Debug)]
pub struct SecretView {
    pub secret_store: SecretStoreView,
    pub updated_at: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {

    use std::env;
    use super::*;
    use common::log::init_logger;

    #[test]
    fn test_db_secret_store() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let secret = SecretStoreView::new_with_specified(
            "0123456789", 1, "key_password", "key_by_answer");
            secret.insert().unwrap();
        let mut secret_by_find = SecretStoreView::find_single(
            SecretFilter::BySittingPubkey("0123456789".to_string())).unwrap();
        println!("{:?}",secret_by_find);
        assert_eq!(secret_by_find.secret_store,secret.secret_store);   

        SecretStoreView::update(
            SecretUpdater::State(SecretKeyState::Deprecated), 
            SecretFilter::BySittingPubkey("01".to_string()), 
        ).unwrap();
    }
}