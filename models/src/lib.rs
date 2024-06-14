//! encapsulation of some postgresql interface for easy call
//#![deny(missing_docs)]
#![deny(warnings)]
//#![allow(unused_imports)]
#![allow(dead_code)]

pub mod account_manager;
pub mod airdrop;
#[macro_use]
pub mod general;
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

use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use common::constants::PG_POOL_SIZE;
use deadpool::managed::Object;
use std::cell::RefCell;
use std::fmt::Display;

use std::sync::Arc;

use deadpool_postgres::Manager;
use deadpool_postgres::Pool;
use deadpool_postgres::Transaction;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use tokio_postgres::Row;

use async_trait::async_trait;

pub type LocalConn = Object<Manager>;

static TRY_TIMES: u8 = 5;

/****

    DBError::RepeatedData,
    DBError::DataNotFound,
    DBError::KeyAlreadyExsit,
*/

lazy_static! {
    pub static ref PG_POOL: Pool = connect_pool().unwrap();
}

tokio::task_local! {
    pub static  LOCAL_CLI: RefCell<Option<Arc<PgLocalCli>>>;
}

pub enum PgLocalCli {
    Conn(&'static mut LocalConn),
    Trans(Transaction<'static>),
}

impl PgLocalCli {
    pub async fn execute(sql: &str) -> Result<u64> {
        debug!(sql);
        let cli = LOCAL_CLI.with(|cli| cli.borrow().as_ref().unwrap().clone());
        let line = match cli.as_ref() {
            PgLocalCli::Conn(c) => c.execute(sql, &[]).await?,
            PgLocalCli::Trans(t) => t.execute(sql, &[]).await?,
        };
        Ok(line)
    }
    pub async fn query(sql: &str) -> Result<Vec<Row>> {
        debug!(sql);
        let cli = LOCAL_CLI.with(|cli| cli.borrow().as_ref().unwrap().clone());
        let row = match cli.as_ref() {
            PgLocalCli::Conn(c) => c.query(sql, &[]).await?,
            PgLocalCli::Trans(t) => t.query(sql, &[]).await?,
        };
        Ok(row)
    }
    pub async fn commit(self) -> Result<()> {
        match self {
            PgLocalCli::Conn(_c) => {
                debug!("it's not a trans");
                Ok(())
            }
            PgLocalCli::Trans(t) => Ok(t.commit().await?),
        }
    }

    pub async fn rollback(self) -> Result<()> {
        match self {
            PgLocalCli::Conn(_c) => {
                debug!("it's not a trans");
                Ok(())
            }
            PgLocalCli::Trans(t) => Ok(t.rollback().await?),
        }
    }

    pub async fn begin(&'static mut self) -> Result<PgLocalCli> {
        match self {
            PgLocalCli::Conn(c) => {
                let trans = c.transaction().await?;
                Ok(PgLocalCli::Trans(trans))
            }
            PgLocalCli::Trans(_t) => {
                panic!("It is already a trans");
            }
        }
    }
}

impl From<&'static mut LocalConn> for PgLocalCli {
    fn from(value: &'static mut LocalConn) -> Self {
        Self::Conn(value)
    }
}

impl From<Transaction<'static>> for PgLocalCli {
    fn from(value: Transaction<'static>) -> Self {
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

    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: PG_POOL_SIZE,
        timeouts: Default::default(),
        queue_mode: Default::default(),
    });

    let pool = cfg.create_pool(None, NoTls).unwrap();
    Ok(pool)
}

#[async_trait]
pub trait PsqlOp {
    type UpdaterContent<'a>: Display + Send;
    type FilterContent<'b>: Display + Send;

    async fn find(filter: Self::FilterContent<'_>) -> Result<Vec<Self>>
    where
        Self: Sized + Send;
    async fn find_single(filter: Self::FilterContent<'_>) -> Result<Self>
    where
        Self: Sized + Send,
    {
        let mut get_res: Vec<Self> = Self::find(filter).await?;
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
    async fn delete(_filter: Self::FilterContent<'_>) -> Result<()> {
        todo!()
    }

    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<u64>;

    async fn update_single(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<()>
    where
        Self: Sized + Send,
    {
        let row_num = Self::update(new_value, filter).await?;
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

    async fn insert(self) -> Result<()>;

    //insert after check key
    async fn safe_insert(self, filter: Self::FilterContent<'_>) -> Result<()>
    where
        Self: Sized + Send,
    {
        let filter_str = filter.to_string();
        let find_res: Vec<Self> = Self::find(filter).await?;
        if find_res.is_empty() {
            self.insert().await
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
    OptionU8(Option<u8>),
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
            PsqlType::OptionU8(data) => {
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

impl From<Option<u8>> for PsqlType {
    fn from(value: Option<u8>) -> Self {
        PsqlType::OptionU8(value)
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
