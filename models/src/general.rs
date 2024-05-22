use std::borrow::BorrowMut;
use std::ops::Deref;
use std::ops::DerefMut;

use anyhow::{Result};
use r2d2::ManageConnection;
use r2d2::PooledConnection;
use r2d2_postgres::postgres::Transaction;
use anyhow::anyhow;
use crate::LocalConnect;
use crate::{PG_POOL,LOCAL_CONN,LOCAL_TX};

pub fn transaction_begin(conn:&mut LocalConnect) -> Result<Transaction> {
   Ok(conn.transaction()?)
}

pub fn transaction_commit(trans:Transaction) -> Result<()> {
    Ok(trans.commit()?)
}

pub fn get_db_pool_connect() -> Result<LocalConnect> {
    let conn =  
        crate::LOCAL_CONN2.take();
    Ok(conn.unwrap())
}


pub fn transaction_rollback() -> Result<u64>{ 
    todo!()
}

pub fn table_clear(table_name: &str) -> Result<(), String> {
    let sql = format!("truncate table {} restart identity", table_name);
    crate::PG_POOL
        .lock()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?
        .execute(sql.as_str(), &[])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn table_all_clear() {
    //table_clear("accounts").unwrap();
    table_clear("users").unwrap();
    table_clear("coin_transaction").unwrap();
    table_clear("device_info").unwrap();
    table_clear("secret_store").unwrap();
    table_clear("ethereum_bridge_order").unwrap();
}
