//! encapsulation of some postgresql interface for easy call
//#![deny(missing_docs)]
//#![deny(warnings)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod account_manager;
pub mod airdrop;
pub mod general;
pub mod newbie_reward;

pub mod coin_transfer;
pub mod secret_store;
//pub mod wallet;
pub mod device_info;
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

use postgres::{Client, NoTls, Row};

use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::sync::Mutex;
use anyhow::Result;
use anyhow::anyhow;

static TRY_TIMES: u8 = 5;

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
    static ref CLIENTDB: Mutex<postgres::Client> = Mutex::new(connect_db().unwrap());
}
fn connect_db() -> Result<Client> {
    let global_conf = &common::env::CONF;
    info!("{}: start postgresql,mode {}", common::utils::time::current_date(),global_conf.service_mode.to_string());
    let url = format!(
        "host=localhost user=postgres port=8068 password=postgres dbname=backend_{}",
        global_conf.service_mode
    );
    let cli = Client::connect(&url, NoTls).map_err(|error| {
        error!("connect postgresql failed,{:?}", error);
        error
    })?;
    Ok(cli)
}

pub fn query(raw_sql: &str) -> Result<Vec<Row>> {
    let mut try_times = TRY_TIMES;
    //todo:
    let mut client = crate::CLIENTDB.lock()
    .map_err(|e|  anyhow!(e.to_string()))?;
    loop {
        debug!("raw_sql {}", raw_sql);
        match client.query(raw_sql, &[]) {
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
                    *client = connect_db()?;
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
}

pub fn execute(raw_sql: &str) -> Result<u64> {
    let mut try_times = TRY_TIMES;
    let mut client = crate::CLIENTDB
        .lock().map_err(|e| anyhow!(e.to_string()))?;
    loop {
        debug!("raw_sql {}", raw_sql);
        match client.execute(raw_sql, &[]) {
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
                    *client = connect_db()?;
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
}

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
            Err(anyhow!("data isn't existed".to_string()))
        } else if data_len > 1 {
            Err(anyhow!("data is repeated".to_string()))
        } else {
            Ok(get_res.pop().unwrap())
        }
    }
    fn delete<T: Display>(_filter: T) -> Result<()> {
        todo!()
    }

    fn update(
        new_value: Self::UpdateContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<()>;

    fn insert(&self) -> Result<()>;
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
