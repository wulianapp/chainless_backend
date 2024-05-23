use std::borrow::BorrowMut;
use std::ops::Deref;
use std::ops::DerefMut;

use anyhow::{Result};
//use r2d2::ManageConnection;
//use r2d2::PooledConnection;
//use r2d2_postgres::postgres::Transaction;
use anyhow::anyhow;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use deadpool_postgres::Transaction;
use crate::LocalConn;
//use crate::PoolConnect;
use crate::{PG_POOL};

/*** 
pub fn transaction_begin_() -> Result<()> {
    LOCAL_CONN.with_borrow_mut(|cn| {
        let transaction = cn.transaction()?;
        LOCAL_TX.with_borrow_mut(|tx|{
            *tx = Some(transaction);
            Ok(())
        })
    })    
}
*/
pub async fn transaction_begin(conn: &mut LocalConn) -> Result<Transaction> {
   Ok(conn.transaction().await?)
}

pub async fn transaction_commit(tx: Transaction<'_>) -> Result<()>{
    Ok(tx.commit().await?)
}

pub async fn get_db_pool_connect() -> Result<LocalConn> {
    Ok(PG_POOL.get().await?)
}


pub fn transaction_rollback() -> Result<u64>{ 
    todo!()
}

pub async fn table_clear(table_name: &str) -> Result<(), String> {
    let sql = format!("truncate table {} restart identity", table_name);
    crate::PG_POOL
        .get()
        .await
        .map_err(|e| e.to_string())?
        .execute(sql.as_str(), &[])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn table_all_clear() {
    //table_clear("accounts").unwrap();
    table_clear("users").await.unwrap();
    table_clear("coin_transaction").await.unwrap();
    table_clear("device_info").await.unwrap();
    table_clear("secret_store").await.unwrap();
    table_clear("ethereum_bridge_order").await.unwrap();
}
