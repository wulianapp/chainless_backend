use std::borrow::BorrowMut;
use std::ops::Deref;
use std::ops::DerefMut;

use anyhow::Result;
//use r2d2::ManageConnection;
//use r2d2::PooledConnection;
//use r2d2_postgres::postgres::Transaction;
use crate::LocalConn;
use crate::PgLocalCli;
use anyhow::anyhow;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use deadpool_postgres::Transaction;
//use crate::PoolConnect;
use crate::PG_POOL;

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

pub async fn transaction_commit(tx: Transaction<'_>) -> Result<()> {
    Ok(tx.commit().await?)
}

pub async fn get_pg_pool_connect<T: From<LocalConn>>() -> Result<T> {
    let conn = PG_POOL.get().await?;
    Ok(conn.into())
}

pub fn transaction_rollback() -> Result<u64> {
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

pub async fn init_system_config() -> Result<(), String> {
    let insert_root_user = "insert into users (
        id,
        phone_number,
        email,
        login_pwd_hash,
        anwser_indexes,
        is_frozen,
        kyc_is_verified,
        create_subacc_time,
        main_account
    ) values (66,NULL,'1@gmail.com','a6666666','answer123',false,true,ARRAY[]::int[],'66.local');";
    let insert_root_airdrop = "insert into airdrop (
        user_id,
        account_id,
        invite_code,
        predecessor_user_id,
        predecessor_account_id,
        btc_address,
        btc_level
        ) values (66,'66.local','chainless.hk','0','0.local','btc_address_abc',0);";
    let cli = crate::PG_POOL.get().await.map_err(|e| e.to_string())?;

    cli.execute(insert_root_user, &[])
        .await
        .map_err(|e| e.to_string())?;
    cli.execute(insert_root_airdrop, &[])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn table_all_clear() {
    table_clear("airdrop").await.unwrap();
    table_clear("users").await.unwrap();
    table_clear("coin_transaction").await.unwrap();
    table_clear("device_info").await.unwrap();
    table_clear("secret_store").await.unwrap();
    table_clear("ethereum_bridge_order").await.unwrap();
    init_system_config().await.unwrap();
}
