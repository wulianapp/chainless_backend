use crate::LocalConn;
use crate::PgLocalCli;
use crate::LOCAL_CLI;
use anyhow::Result;
use futures::Future;
use std::cell::RefCell;
use std::sync::Arc;

use anyhow::anyhow;

use crate::PG_POOL;

pub async fn gen_db_cli(method: &str) -> Result<(PgLocalCli, *mut LocalConn)> {
    let conn = PG_POOL.get().await?;
    debug!("pool_status {:?}", PG_POOL.status());
    let conn = Box::new(conn);
    let conn: &'static mut LocalConn = Box::leak(conn);
    let conn_ptr = conn as *mut LocalConn;

    let db_cli = match method {
        "POST" => {
            let trans = conn.transaction().await?;
            PgLocalCli::Trans(trans)
        }
        _ => PgLocalCli::Conn(conn),
    };
    Ok((db_cli, conn_ptr))
}

/*** 
pub async fn clean_db_cli(conn_ptr: *mut LocalConn) -> Result<()> {
    let db_cli = LOCAL_CLI.with(|db_cli| -> Result<PgLocalCli> {
        let mut db_cli = db_cli.borrow_mut();
        let db_cli = db_cli.take().ok_or(anyhow!(""))?;
        let db_cli = Arc::into_inner(db_cli).ok_or(anyhow!(""))?;
        Ok(db_cli)
    })?;

    db_cli.commit().await?;
    unsafe {
        let _ = Box::from_raw(conn_ptr);
    };
    Ok(())
}
***/


pub fn clean_conn(conn_ptr: *mut LocalConn) {
    unsafe {
        let _ = Box::from_raw(conn_ptr);
    };
    debug!("pool_status {:?}", PG_POOL.status());
}

pub fn into_local_cli() -> Result<PgLocalCli>{
    LOCAL_CLI.with(|db_cli| -> Result<PgLocalCli> {
        let mut db_cli = db_cli.borrow_mut();
        let db_cli = db_cli.take().ok_or(anyhow!(""))?;
        let db_cli = Arc::into_inner(db_cli).ok_or(anyhow!(""))?;
        Ok(db_cli)
    })
}

pub async fn commit() -> Result<()> {
    let db_cli = into_local_cli()?;
    db_cli.commit().await?;
    Ok(())
}

pub async fn rollback() -> Result<()> {
    let db_cli = into_local_cli()?;
    db_cli.rollback().await?;
    Ok(())
}

pub async fn run_api_call<Fut, R>(method: &str, task: Fut) -> Result<(*mut LocalConn,R)>
where
    Fut: Future<Output = R> + 'static,
{
    let (db_cli, conn_ptr) = gen_db_cli(method).await?;
    let res = crate::LOCAL_CLI
        .scope(RefCell::new(Some(Arc::new(db_cli))), async move {
            task.await
        })
        .await;
    Ok((conn_ptr,res))
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
        main_account,
        token_version
    ) values (66,NULL,'1@gmail.com','a6666666','answer123',false,true,'66.local',1);";
    let insert_root_airdrop = "insert into airdrop (
        user_id,
        account_id,
        invite_code,
        predecessor_user_id,
        predecessor_account_id,
        btc_address,
        btc_level,
        btc_grade_status
        ) values (66,'66.local','chainless.hk','0','0.local','btc_address_abc',0,'Reconfirmed');";
    let conn = crate::PG_POOL.get().await.map_err(|e| e.to_string())?;

    conn.execute(insert_root_user, &[])
        .await
        .map_err(|e| e.to_string())?;
    conn.execute(insert_root_airdrop, &[])
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
    table_clear("wallet_manage_record").await.unwrap();
    init_system_config().await.unwrap();
}
