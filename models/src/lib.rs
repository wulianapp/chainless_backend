//! encapsulation of some postgresql interface for easy call
//#![deny(missing_docs)]
//#![deny(warnings)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod account_manager;
pub mod airdrop;
#[macro_use]
pub mod general;
pub mod newbie_reward;

pub mod coin_transfer;
pub mod device_info;
pub mod eth_bridge_order;
pub mod secret_store;
pub mod wallet_manage_record;

//#[macro_use]
//extern crate log;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate rustc_serialize;
extern crate tokio_postgres;

//use postgres::{Client, NoTls, Row};

use anyhow::anyhow;
use anyhow::Result;
use deadpool::managed::Object;
use general::get_pg_pool_connect;
//use r2d2_postgres::postgres::GenericClient;
//use r2d2_postgres::postgres::Transaction;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::future;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio_postgres::NoTls;
use tokio_postgres::Row;
//use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
//use r2d2::Pool;
//use r2d2_postgres::postgres::Row;
use deadpool_postgres::Manager;
use deadpool_postgres::Pool;
use deadpool_postgres::Transaction;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod};

use async_trait::async_trait;
use tokio::runtime::Runtime;
//use futures::executor::block_on;
use async_std::task::block_on;

use ouroboros::self_referencing;

//type PoolConnect = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;
type LocalConn = Object<Manager>;

static TRY_TIMES: u8 = 5;

/****

    DBError::RepeatedData,
    DBError::DataNotFound,
    DBError::KeyAlreadyExsit,
*/

lazy_static! {
    static ref PG_POOL: Pool = connect_pool().unwrap();
}

/***
thread_local! {
    pub static  LOCAL_CONN: RefCell<LocalConn> = {
        let conn = block_on(async{
            error!("Gen LOCAL_CONN ");
            PG_POOL.get().await.unwrap()
        });
        RefCell::new(conn)
    };

    pub static  LOCAL_CONN2: LocalConn = {
        let conn = block_on(async{
            error!("Gen LOCAL_CONN ");
            PG_POOL.get().await.unwrap()
        });
        conn
    };

    pub static LOCAL_TX: RefCell<Option<Transaction<'static>>> = RefCell::new(None);

    pub static LOCAL_CONN4: (Pool,Option<Transaction<'static>>) = {
        unimplemented!()
    };

}
**/

pub enum PgLocalCli<'a> {
    Conn(LocalConn),
    Trans(Transaction<'a>),
}

/***
struct DBCli<'a,T: PsqlOp>{
    pg_cli: PgLocalCli<'a>,
    table: T
}
**/

impl PgLocalCli<'_> {
    pub async fn execute(&mut self, sql: &str) -> Result<u64> {
        let line = match self {
            PgLocalCli::Conn(c) => c.execute(sql, &[]).await?,
            PgLocalCli::Trans(t) => t.execute(sql, &[]).await?,
        };
        Ok(line)
    }
    pub async fn query(&mut self, sql: &str) -> Result<Vec<Row>> {
        let row = match self {
            PgLocalCli::Conn(c) => c.query(sql, &[]).await?,
            PgLocalCli::Trans(t) => t.query(sql, &[]).await?,
        };
        Ok(row)
    }
    pub async fn commit(self) -> Result<()> {
        match self {
            PgLocalCli::Conn(_c) => {
                panic!("it's not a trans")
            }
            PgLocalCli::Trans(t) => Ok(t.commit().await?),
        }
    }

    pub async fn begin(&mut self) -> Result<PgLocalCli<'_>> {
        match self {
            PgLocalCli::Conn(c) => {
                let trans = c.transaction().await?;
                Ok(PgLocalCli::Trans(trans))
            }
            PgLocalCli::Trans(_t) => {
                panic!("It is already a trans")
            }
        }
    }
}

impl<'a> From<LocalConn> for PgLocalCli<'a> {
    fn from(value: LocalConn) -> Self {
        Self::Conn(value)
    }
}

impl<'a> From<Transaction<'a>> for PgLocalCli<'a> {
    fn from(value: Transaction<'a>) -> Self {
        Self::Trans(value)
    }
}

fn connect_pool() -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.dbname = Some(common::env::CONF.database.dbname.clone());
    cfg.user = Some(common::env::CONF.database.user.clone());
    cfg.password = Some(common::env::CONF.database.password.clone());
    cfg.host = Some(common::env::CONF.database.host.clone());
    cfg.port = Some(common::env::CONF.database.port as u16);

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    //let manager = Manager::new(common::env::CONF.database.db_uri().as_str(), NoTls2);
    //let pool = Pool::builder(manager).max_size(16).build().unwrap();
    let pool = cfg.create_pool(None, NoTls).unwrap();
    Ok(pool)
}

/***
pub async fn query(raw_sql: &str,cli: &mut PgLocalCli<'_>) -> Result<Vec<Row>> {
    let mut try_times = TRY_TIMES;
    //let mut x = get_db_pool_connect().await?;
    let res = cli.query(raw_sql).await?;
    Ok(res)

}

pub async fn execute(raw_sql: String) -> Result<u64> {
    let mut try_times = TRY_TIMES;
    /***
    let local_conn = LOCAL_CONN2.with(|x| {
        x.clone()
    });
    loop {
        error!("_0003_finish3 execute ");
        let tmp1 = local_conn.lock().unwrap();
        match tmp1.execute(&raw_sql, &[]).await{
            Ok(data) => {
                return Ok(data);
            }
            Err(error) => {
                if try_times == 0 {
                    let error_info = format!("erro:{:?}, query still failed after retry", error);
                    error!("{}", error_info);
                    Err(anyhow!(error_info))?;
                } else {
                    error!("error {:?}", error);
                    //crate::PG_POOL = connect_pool()?;
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
    **/
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    Ok(0)

}

pub async fn query_with_trans(raw_sql: &str,tx: &mut Transaction<'_>) -> Result<Vec<Row>> {
    Ok(tx.query(raw_sql, &[]).await?)
  }


pub async fn execute_with_trans(raw_sql: &str,tx: &mut Transaction<'_>) -> Result<u64> {
    Ok(tx.execute(raw_sql, &[]).await?)
}

pub fn execute2(raw_sql: &str) -> Result<u64> {
    let mut try_times = TRY_TIMES;
    let mut pg_client = crate::CLIENTDB.lock().map_err(|e| anyhow!(e.to_string()))?;
    //let mut pg_client2 = LOCAL_CLI.take();
    LOCAL_CLI.with_borrow_mut(|client|{
        Ok(client.execute(raw_sql, &[])?)
    })
}
***/

type UpdaterContent = String;
type FilterContent = String;
#[async_trait]
pub trait PsqlOp {
    type UpdaterContent<'a>: Display + Send;
    type FilterContent<'b>: Display + Send;

    async fn find(filter: Self::FilterContent<'_>, cli: &mut PgLocalCli<'_>) -> Result<Vec<Self>>
    where
        Self: Sized + Send;
    async fn find_single(filter: Self::FilterContent<'_>, cli: &mut PgLocalCli<'_>) -> Result<Self>
    where
        Self: Sized + Send,
    {
        let mut get_res: Vec<Self> = Self::find(filter, cli).await?;
        let data_len = get_res.len();
        if data_len == 0 {
            //todo:return db error type
            let error_info = "DBError::DataNotFound: data isn't existed";
            error!("{}", error_info);
            Err(anyhow!(error_info.to_string()))
        } else if data_len > 1 {
            let error_info = "DBError::RepeatedData: data is repeated";
            error!("{}", error_info);
            Err(anyhow!(error_info.to_string()))
        } else {
            Ok(get_res.pop().unwrap())
        }
    }
    async fn delete(_filter: Self::FilterContent<'_>, _cli: &mut PgLocalCli<'_>) -> Result<()> {
        todo!()
    }

    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<u64>;

    async fn update_single(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<()>
    where
        Self: Sized + Send,
    {
        let row_num = Self::update(new_value, filter, cli).await?;
        if row_num == 0 {
            //todo:return db error type
            let error_info = "DBError::DataNotFound: data isn't existed";
            error!("{}", error_info);
            Err(anyhow!(error_info.to_string()))
        } else if row_num > 1 {
            let error_info = "DBError::RepeatedData: data is repeated";
            error!("{}", error_info);
            Err(anyhow!(error_info.to_string()))
        } else {
            Ok(())
        }
    }

    async fn insert(&self, cli: &mut PgLocalCli<'_>) -> Result<()>;

    //insert after check key
    async fn safe_insert(
        &self,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<()>
    where
        Self: Sized + Send,
    {
        let filter_str = filter.to_string();
        let find_res: Vec<Self> = Self::find(filter, cli).await?;
        if find_res.is_empty() {
            self.insert(cli).await
        } else {
            //let error_info = "DBError::KeyAlreadyExsit: key already existed";
            //error!("{}", error_info);
            //Err(anyhow!(error_info.to_string()))
            info!("data {} already exist", filter_str);
            Ok(())
        }
    }
}

pub trait FormatSql {
    fn string4sql(&self) -> String;
}

impl FormatSql for String {
    fn string4sql(&self) -> String {
        format!("'{}'", self)
    }
}

fn assembly_insert_values(lines: Vec<Vec<String>>) -> String {
    let mut lines_str = "".to_string();
    let mut index = 0;
    let len = lines.len();
    for line in lines {
        let mut line_str = "".to_string();
        for i in 0..line.len() {
            if i < line.len() - 1 {
                line_str = format!("{}{},", line_str, line[i]);
            } else {
                line_str = format!("{}{}", line_str, line[i]);
            }
        }
        if index < len - 1 {
            lines_str = format!("{}{}),(", lines_str, line_str);
        } else {
            lines_str = format!("{}{})", lines_str, line_str);
        }
        index += 1;
    }
    lines_str
}

pub fn vec_str2array_text(vec: Vec<String>) -> String {
    let array_elements: Vec<String> = vec
        .into_iter()
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .collect();

    format!("ARRAY[{}]::text[]", array_elements.join(","))
}

pub enum PsqlType {
    VecStr(Vec<String>),
    VecU64(Vec<u64>),
    OptionStr(Option<String>),
    OptionU64(Option<u64>),
}

impl PsqlType {
    pub fn to_psql_str(&self) -> String {
        match self {
            PsqlType::VecStr(data) => {
                let array_elements: Vec<String> = data
                    .iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect();

                format!("ARRAY[{}]::text[]", array_elements.join(","))
            }
            PsqlType::VecU64(data) => {
                let array_elements: Vec<String> = data.iter().map(|s| format!("{}", s)).collect();

                format!("ARRAY[{}]::int4[]", array_elements.join(","))
            }
            PsqlType::OptionStr(data) => data
                .to_owned()
                .map(|x| format!("'{}'", x))
                .unwrap_or("NULL".to_string()),
            PsqlType::OptionU64(data) => {
                data.map(|x| format!("{}", x)).unwrap_or("NULL".to_string())
            }
        }
    }
}

impl From<Vec<String>> for PsqlType {
    fn from(value: Vec<String>) -> Self {
        PsqlType::VecStr(value)
    }
}

impl From<Vec<u64>> for PsqlType {
    fn from(value: Vec<u64>) -> Self {
        PsqlType::VecU64(value)
    }
}

impl From<Option<String>> for PsqlType {
    fn from(value: Option<String>) -> Self {
        PsqlType::OptionStr(value)
    }
}

impl From<Option<u64>> for PsqlType {
    fn from(value: Option<u64>) -> Self {
        PsqlType::OptionU64(value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
