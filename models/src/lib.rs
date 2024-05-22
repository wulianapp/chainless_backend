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
pub mod secret_store;
//pub mod wallet;
pub mod device_info;
pub mod eth_bridge_order;
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
extern crate postgres;
extern crate rustc_serialize;

use anyhow::anyhow;
use anyhow::Result;
use r2d2_postgres::postgres::GenericClient;
use r2d2_postgres::postgres::Transaction;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Mutex;
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;
use r2d2_postgres::postgres::Row;
use ouroboros::self_referencing;

type LocalConnect = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;
type GlobalPool = Pool<PostgresConnectionManager<NoTls>>;
static TRY_TIMES: u8 = 5;

/****

    DBError::RepeatedData,
    DBError::DataNotFound,
    DBError::KeyAlreadyExsit,
*/

///time limit scope
#[derive(Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum TimeScope {
    NoLimit,
    SevenDay,
    OneDay,
}

impl TimeScope {
    // scope filter
    pub fn filter_str(&self) -> &'static str {
        match self {
            TimeScope::NoLimit => "",
            TimeScope::SevenDay => "where created_at > NOW() - INTERVAL '7 day'",
            TimeScope::OneDay => "where created_at > NOW() - INTERVAL '24 hour'",
        }
    }
}


lazy_static! {
    static ref PG_POOL: Mutex<GlobalPool> = {
        Mutex::new(connect_pool().unwrap())
    };
}
//todo: set global Transaction 



thread_local! {
    pub static LOCAL_CONN: RefCell<Option<LocalConnect>> = {
        RefCell::new(Some(PG_POOL.lock().unwrap().get().unwrap()))
    };

    pub static LOCAL_CONN2: RefCell<Option<LocalConnect>> = {
        RefCell::new(Some(PG_POOL.lock().unwrap().get().unwrap()))
    };

    pub static LOCAL_TX: RefCell<Option<Transaction<'static>>> = RefCell::new(None);
  
}

/*** 
pub enum PgLocalCli<'a,'b,'c> {
    Cli(&'b PoolConnect),
    Tx(&'c Transaction<'a>)
}

impl PgLocalCli<'_,'_,'_> {
    pub fn execute(&mut self,sql:&str) -> Result<u64>{
        let line = match self {
            PgLocalCli::Cli(c) => {
                c.execute(sql, &[])?
            },
            PgLocalCli::Tx(t) => {
                t.execute(sql, &[])?
            },
        };
        Ok(line)
    }
    pub fn query(&mut self,sql:&str) -> Result<Vec<Row>>{
        let row = match self {
            PgLocalCli::Cli(c) => {
                c.query(sql, &[])?
            },
            PgLocalCli::Tx(t) => {
                t.query(sql, &[])?
            },
        };
        Ok(row)
    }
    pub fn commit(mut self) -> Result<()>{
        match self {
            PgLocalCli::Cli(c) => {
                debug!("as a connet no nothing");
                Ok(())
            },
            PgLocalCli::Tx(t) => {
                Ok(t.commit()?)
            },
        }
    }
}

impl<'b> From<&'b PoolConnect> for PgLocalCli<'_,'b,'_>{
    fn from(value: &'b PoolConnect) -> Self {
        Self::Cli(value)
    }
}

impl<'a,'c> From<&'c Transaction<'a>> for PgLocalCli<'a,'_,'c>{
    fn from(value: &'c Transaction<'a>) -> Self {
        Self::Tx(value)
    }
}
*/

fn connect_pool() -> Result<GlobalPool>{
    let manager = PostgresConnectionManager::new(
        common::env::CONF.database.db_uri().parse().unwrap(),
        NoTls,
    );
    let pool = r2d2::Pool::new(manager).unwrap();
    Ok(pool)
}

pub fn query(raw_sql: &str) -> Result<Vec<Row>> {
    let mut try_times = TRY_TIMES;
    /*** 
    let cli: &mut PgLocalCli =  LOCAL_TX.with_borrow_mut(|x|{
        match *x {
            Some(tx) => &mut tx.into(),
            None => LOCAL_CONN.with_borrow_mut(|x|{
                &mut x.into()
            })
        }
    });
    */
    LOCAL_CONN.with_borrow_mut(|x|{
        loop {
            debug!("raw_sql {}", raw_sql);
            match x.as_mut().unwrap().query(raw_sql,&[]) {
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
                        let mut pool = crate::PG_POOL.lock().map_err(|e| anyhow!(e.to_string()))?;
                        *pool = connect_pool()?;
                        try_times -= 1;
                        continue;
                    }
                }
            }
        }
    })
    
}


pub fn query_with_trans(raw_sql: &str,tx: &mut Transaction) -> Result<Vec<Row>> {
  Ok(tx.query(raw_sql, &[])?)
}


pub fn execute_with_trans(raw_sql: &str,tx: &mut Transaction) -> Result<u64> {
    Ok(tx.execute(raw_sql, &[])?)
}


pub fn execute(raw_sql: &str) -> Result<u64> {
    let mut try_times = TRY_TIMES;

    /*** 
    let cli: &mut PgLocalCli =  LOCAL_TX.with_borrow_mut(|x|{
        match *x {
            Some(tx) => &mut tx.into(),
            None => LOCAL_CONN.with_borrow_mut(|x|{
                &mut x.unwrap().into()
            })
        }
    });
    **/
    LOCAL_CONN.with_borrow_mut(|x|{
        loop {
            debug!("raw_sql {}", raw_sql);
            match x.as_mut().unwrap().execute(raw_sql,&[]) {
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
                        let mut pool = crate::PG_POOL.lock().map_err(|e| anyhow!(e.to_string()))?;
                        *pool = connect_pool()?;
                        try_times -= 1;
                        continue;
                    }
                }
            }
        }
    })
 
}

/*** 
pub fn execute2(raw_sql: &str) -> Result<u64> {
    let mut try_times = TRY_TIMES;
    let mut client = crate::CLIENTDB.lock().map_err(|e| anyhow!(e.to_string()))?;
    //let mut client2 = LOCAL_CLI.take();
    LOCAL_CLI.with_borrow_mut(|client|{
        Ok(client.execute(raw_sql, &[])?)
    })
}
***/

pub trait PsqlOp {
    type UpdateContent<'a>: Display;
    type FilterContent<'b>: Display;
    fn find(filter: Self::FilterContent<'_>) -> Result<Vec<Self>>
    where
        Self: Sized;
    fn find_single(filter: Self::FilterContent<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut get_res: Vec<Self> = Self::find(filter)?;
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
    fn delete<T: Display>(_filter: T) -> Result<()> {
        todo!()
    }

    fn update(new_value: Self::UpdateContent<'_>, 
        filter: Self::FilterContent<'_>
    ) -> Result<u64>;

    fn update_with_trans(new_value: Self::UpdateContent<'_>, 
        filter: Self::FilterContent<'_>,
        trans:&mut Transaction
    ) -> Result<u64>;


    fn update_single(
        new_value: Self::UpdateContent<'_>,
        filter: Self::FilterContent<'_>
    ) -> Result<()>
    where
        Self: Sized,
    {
        let row_num = Self::update(new_value, filter)?;
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

    fn update_single_with_trans(
        new_value: Self::UpdateContent<'_>,
        filter: Self::FilterContent<'_>,
        trans: &mut Transaction
    ) -> Result<()>
    where
        Self: Sized,
    {
        let row_num = Self::update_with_trans(new_value, filter,trans)?;
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

    fn insert(&self) -> Result<()>;

    fn insert_with_trans(&self,trans: &mut Transaction) -> Result<()>;


    //insert after check key
    fn safe_insert(&self, filter: Self::FilterContent<'_>) -> Result<()>
    where
        Self: Sized,
    {
        let filter_str = filter.to_string();
        let find_res: Vec<Self> = Self::find(filter)?;
        if find_res.is_empty() {
            self.insert()
        } else {
            //let error_info = "DBError::KeyAlreadyExsit: key already existed";
            //error!("{}", error_info);
            //Err(anyhow!(error_info.to_string()))
            info!("data {} already exist", filter_str);
            Ok(())
        }
    }

    //insert after check key
    fn safe_insert_with_trans(&self, filter: Self::FilterContent<'_>,trans:&mut Transaction) -> Result<()>
    where
        Self: Sized,
    {
        let filter_str = filter.to_string();
        let find_res: Vec<Self> = Self::find(filter)?;
        if find_res.is_empty() {
            self.insert_with_trans(trans)
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
