//! encapsulation of some postgresql interface for easy call
//#![deny(missing_docs)]
//#![deny(warnings)]

pub mod account_manager;
pub mod airdrop;
pub mod general;
pub mod newbie_reward;

pub mod coin_transfer;
pub mod wallet;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate postgres;
extern crate rustc_serialize;

use postgres::{Client, NoTls, Row};

use anyhow::anyhow;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Mutex;

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

///restart postgres client
pub fn gen_new_client() -> Client {
    let now = Local::now();
    println!("restart postgresql {:?}", now);
    connect_db().unwrap()
}

fn connect_db() -> Option<postgres::Client> {
    let global_conf = &common::env::CONF;
    let url = format!(
        "host=localhost user=postgres port=5432 password=postgres dbname=backend_{}",
        global_conf.service_mode.to_string()
    );

    match Client::connect(&url, NoTls) {
        Ok(client) => {
            eprintln!("connect postgresql successfully");
            Some(client)
        }
        Err(error) => {
            eprintln!("connect postgresql failed,{:?}", error);
            None
        }
    }
}

pub fn query(raw_sql: &str) -> anyhow::Result<Vec<Row>> {
    let mut try_times = TRY_TIMES;
    let mut client = crate::CLIENTDB.lock().unwrap();
    loop {
        println!("raw_sql {}", raw_sql);
        match client.query(raw_sql, &[]) {
            Ok(data) => {
                return Ok(data);
            }
            Err(error) => {
                if try_times == 0 {
                    //Err(anyhow!("Missing attribute: {}", missing));
                    return Err(anyhow!("retry query failed"));
                } else {
                    error!("error {:?}", error);
                    println!("error {:?}", error);
                    *client = crate::gen_new_client();
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
}

pub fn execute(raw_sql: &str) -> anyhow::Result<u64> {
    let mut try_times = TRY_TIMES;
    let mut client = crate::CLIENTDB.lock().unwrap();
    loop {
        println!("raw_sql {}", raw_sql);
        match client.execute(raw_sql, &[]) {
            Ok(data) => {
                return Ok(data);
            }
            Err(error) => {
                if try_times == 0 {
                    return Err(anyhow!("retry execute failed"));
                } else {
                    error!("error {:?}", error);
                    println!("error {:?}", error);
                    *client = crate::gen_new_client();
                    try_times -= 1;
                    continue;
                }
            }
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
        .map(|s| format!("'{}'", s.replace("'", "''")))
        .collect();

    format!("ARRAY[{}]::text[]", array_elements.join(","))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
