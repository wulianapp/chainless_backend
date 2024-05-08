use anyhow::Result;

pub fn transaction_begin() -> Result<(), String> {
    crate::CLIENTDB
        .lock()
        .map_err(|e| e.to_string())?
        .simple_query("begin")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn transaction_commit() -> Result<(), String> {
    crate::CLIENTDB
        .lock()
        .map_err(|e| e.to_string())?
        .simple_query("commit")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn transaction_rollback() -> Result<(), String> {
    crate::CLIENTDB
        .lock()
        .map_err(|e| e.to_string())?
        .simple_query("rollback")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn table_clear(table_name: &str) -> Result<(), String> {
    let sql = format!("truncate table {} restart identity", table_name);
    crate::CLIENTDB
        .lock()
        .map_err(|e| e.to_string())?
        .execute(sql.as_str(), &[])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn select_db(db_name: &str) -> Result<(), String> {
    //fixme: 数据会丢,另外切数据库失败
    let sql = format!("\\c {}", db_name);
    crate::CLIENTDB
        .lock()
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
