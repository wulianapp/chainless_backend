extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::SecretKeyState;
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;
use std::fmt;
use std::fmt::Display;
use tokio_postgres::Row;

use crate::{vec_str2array_text, PgLocalCli, PsqlOp};
use anyhow::{Ok, Result};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SecretStoreEntity {
    pub secret_store: SecretStore,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Debug)]
pub enum SecretUpdater<'a> {
    //todo:
    EncrypedPrikey(&'a str, &'a str),
    State(SecretKeyState),
}

impl fmt::Display for SecretUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretUpdater::EncrypedPrikey(by_password, by_answer) => {
                format!(
                    "(encrypted_prikey_by_password,encrypted_prikey_by_answer)=('{}','{}')",
                    by_password, by_answer
                )
            }
            SecretUpdater::State(new_state) => {
                format!("state='{}'", new_state)
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum SecretFilter<'b> {
    ByPubkey(&'b str),
    BySittingPubkey(&'b str),
}

impl fmt::Display for SecretFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SecretFilter::ByPubkey(key) => format!("pubkey='{}' ", key),
            SecretFilter::BySittingPubkey(key) => format!("state='Sitting' and pubkey='{}' ", key),
        };
        write!(f, "{}", description)
    }
}

impl SecretStoreEntity {
    pub fn new_with_specified(
        pubkey: &str,
        user_id: u32,
        encrypted_prikey_by_password: &str,
        encrypted_prikey_by_answer: &str,
    ) -> Self {
        SecretStoreEntity {
            secret_store: SecretStore {
                pubkey: pubkey.to_string(),
                state: SecretKeyState::Incumbent,
                user_id,
                encrypted_prikey_by_password: encrypted_prikey_by_password.to_string(),
                encrypted_prikey_by_answer: encrypted_prikey_by_answer.to_string(),
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}
#[async_trait]
impl PsqlOp for SecretStoreEntity {
    type UpdaterContent<'a> = SecretUpdater<'a>;
    type FilterContent<'b> = SecretFilter<'b>;
    async fn find(
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<Vec<SecretStoreEntity>> {
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
            filter
        );
        let execute_res = cli.query(sql.as_str()).await?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row| {
            Ok(SecretStoreEntity {
                secret_store: SecretStore {
                    pubkey: row.get(0),
                    state: row.get::<usize, String>(1).parse()?,
                    user_id: row.get::<usize, i32>(2) as u32,
                    encrypted_prikey_by_password: row.get(3),
                    encrypted_prikey_by_answer: row.get(4),
                },
                updated_at: row.get(4),
                created_at: row.get(5),
            })
        };

        execute_res.iter().map(gen_view).collect()
    }
    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update secret_store set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = cli.execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(&self, cli: &mut PgLocalCli<'_>) -> Result<()> {
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
            pubkey, state, user_id, encrypted_prikey_by_password, encrypted_prikey_by_answer
        );
        debug!("row sql {} rows", sql);
        let _execute_res = cli.execute(sql.as_str()).await?;
        Ok(())
    }

    async fn delete(_filter: Self::FilterContent<'_>, _cli: &mut PgLocalCli<'_>) -> Result<()> {
        todo!()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretView {
    pub secret_store: SecretStoreEntity,
    pub updated_at: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {

    use super::*;
    use common::log::init_logger;
    use std::env;

    /***
    #[tokio::test]
    async fn test_db_secret_store() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let secret =
            SecretStoreView::new_with_specified("0123456789", 1, "key_password", "key_by_answer");
        secret.insert().await.unwrap();
        let secret_by_find =
            SecretStoreView::find_single(SecretFilter::BySittingPubkey("0123456789")).await.unwrap();
        println!("{:?}", secret_by_find);
        assert_eq!(secret_by_find.secret_store, secret.secret_store);

        SecretStoreView::update(
            SecretUpdater::State(SecretKeyState::Abandoned),
            SecretFilter::BySittingPubkey("01"),
        ).await
        .unwrap();
    }
    **/
}
