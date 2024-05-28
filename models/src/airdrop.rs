
extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::airdrop::Airdrop;
use common::data_structures::SecretKeyState;
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;
use std::fmt;
use std::fmt::Display;
use tokio_postgres::Row;

use crate::{vec_str2array_text, PgLocalCli, PsqlOp, PsqlType};
use anyhow::{Ok, Result};

#[derive(Debug)]
pub enum AirdropUpdater<'a> {
    InviteCode(&'a str),
    BtcAddress(&'a str),
    AccountId(&'a str),
    //user_id,account_id
    Predecessor(&'a str,&'a str),
    BtcLevel(u8)
}

impl fmt::Display for AirdropUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AirdropUpdater::InviteCode(code) => {
                format!("invite_code='{}'", code)
            },
            AirdropUpdater::BtcAddress(addr) => {
                format!("btc_address='{}'", addr)
            },
            AirdropUpdater::AccountId(id) => {
                format!("account_id='{}'", id)
            },
            AirdropUpdater::Predecessor(user_id,account_id) => {
                format!("predecessor_user_id='{}',predecessor_account_id='{}'",user_id,account_id)
            },
            AirdropUpdater::BtcLevel(level) => {
                format!("btc_level='{}'", level)
            },
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum AirdropFilter<'b> {
    ByInviteCode(&'b str),
    ByAccountId(&'b str),
    ByBtcAddress(&'b str),
    ByUserId(&'b str),
}

impl fmt::Display for AirdropFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AirdropFilter::ByInviteCode(code) => format!("invite_code='{}' ", code),
            AirdropFilter::ByAccountId(id) => format!("account_id='{}' ", id),
            AirdropFilter::ByUserId(id) => format!("user_id='{}' ", id),
            AirdropFilter::ByBtcAddress(addr) => format!("btc_address='{}' ", addr),
        };
        write!(f, "{}", description)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AirdropView {
    pub airdrop: Airdrop,
    pub updated_at: String,
    pub created_at: String,
}

impl AirdropView {
    pub fn new_with_specified(
        user_id: &str,
        predecessor_user_id: &str,
        predecessor_account_id: &str,
    ) -> Self {
        AirdropView {
            airdrop: Airdrop {
                user_id:user_id.to_string(),
                account_id: None,
                invite_code: user_id.to_string(),
                predecessor_user_id: predecessor_user_id.to_string(),
                predecessor_account_id: predecessor_account_id.to_string(),
                btc_address: None,
                btc_level: None,
                airdrop_reserved_field1: "".to_string(),
                airdrop_reserved_field2:  "".to_string(),
                airdrop_reserved_field3:  "".to_string(),
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}
#[async_trait]
impl PsqlOp for AirdropView {
    type UpdaterContent<'a> = AirdropUpdater<'a>;
    type FilterContent<'b> = AirdropFilter<'b>;
    async fn find(
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<Vec<AirdropView>> {
        let sql = format!(
            "select 
            user_id,\
            account_id,\
            invite_code,\
            predecessor_user_id,\
            predecessor_account_id,\
            btc_address,\
            btc_level,\
            airdrop_reserved_field1,\
            airdrop_reserved_field2,\
            airdrop_reserved_field3,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from airdrop where {}",
            filter
        );
        let execute_res = cli.query(sql.as_str()).await?;
        debug!("get_airdrop: raw sql {}", sql);
        let gen_view = |row: &Row| {
            Ok(AirdropView {
                airdrop: Airdrop { 
                    user_id: row.get(0), 
                    account_id: row.get::<usize, Option<String>>(1),
                    invite_code: row.get(2), 
                    predecessor_user_id: row.get(3), 
                    predecessor_account_id: row.get::<usize, String>(4), 
                    btc_address: row.get::<usize, Option<String>>(5), 
                    btc_level: row.get::<usize, Option<i16>>(6).map(|x| x as u8), 
                    airdrop_reserved_field1: row.get(7), 
                    airdrop_reserved_field2: row.get(8), 
                    airdrop_reserved_field3: row.get(9)
                },
                updated_at: row.get(10),
                created_at: row.get(11),
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
            "update airdrop set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = cli.execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(&self, cli: &mut PgLocalCli<'_>) -> Result<()> {
        let Airdrop { 
            user_id, 
            account_id, 
            invite_code, 
            predecessor_user_id, 
            predecessor_account_id,
            btc_address, 
            btc_level, 
            airdrop_reserved_field1, 
            airdrop_reserved_field2, 
            airdrop_reserved_field3 
        } = self.airdrop.clone();
        let account_id: PsqlType = account_id.into();
        let btc_address: PsqlType = btc_address.into();
        let btc_level: PsqlType = btc_level.into();

        let sql = format!(
            "insert into airdrop (\
                user_id,\
                account_id,\
                invite_code,\
                predecessor_user_id,\
                predecessor_account_id,\
                btc_address,\
                btc_level,\
                airdrop_reserved_field1,\
                airdrop_reserved_field2,\
                airdrop_reserved_field3
         ) values ('{}',{},'{}','{}','{}',{},{},'{}','{}','{}');",
            user_id, 
            account_id.to_psql_str(),
            invite_code,
            predecessor_user_id,
            predecessor_account_id,
            btc_address.to_psql_str(),
            btc_level.to_psql_str(),
            airdrop_reserved_field1,
            airdrop_reserved_field2,
            airdrop_reserved_field3,
        );
        debug!("row sql {} rows", sql);
        let _execute_res = cli.execute(sql.as_str()).await?;
        Ok(())
    }

    async fn delete(_filter: Self::FilterContent<'_>, _cli: &mut PgLocalCli<'_>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::general::get_pg_pool_connect;

    use super::*;
    use common::log::init_logger;
    use std::env;

    #[tokio::test]
    async fn test_db_airdop() {
        env::set_var("CONFIG", "/root/chainless_backend/config_test.toml");
        init_logger();
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();

        let airdrop =
            AirdropView::new_with_specified("1", "2","3.local");
        airdrop.insert(&mut db_cli).await.unwrap();
        let airdrop_by_find =
            AirdropView::find_single(AirdropFilter::ByInviteCode("1"),&mut db_cli).await.unwrap();
        println!("{:?}", airdrop_by_find);
        assert_eq!(airdrop.airdrop, airdrop_by_find.airdrop);

        AirdropView::update_single(
            AirdropUpdater::InviteCode("3"),
            AirdropFilter::ByInviteCode("1"),
            &mut db_cli
        ).await
        .unwrap();
    }
}
