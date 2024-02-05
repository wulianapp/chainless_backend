use common::error_code::BackendError;
use common::error_code::BackendError::{DBError, InternalError};

pub fn transaction_begin() -> Result<(), BackendError> {
    crate::CLIENTDB
        .lock()
        .map_err(|e| InternalError(e.to_string()))?
        .simple_query("begin")
        .map_err(|e| DBError(e.to_string()))?;
    Ok(())
}

pub fn transaction_commit() -> Result<(), BackendError> {
    crate::CLIENTDB
        .lock()
        .map_err(|e| InternalError(e.to_string()))?
        .simple_query("commit")
        .map_err(|e| DBError(e.to_string()))?;
    Ok(())
}

pub fn table_clear(table_name: &str) -> Result<(), BackendError> {
    let sql = format!("truncate table {} restart identity", table_name);
    crate::CLIENTDB
        .lock()
        .map_err(|e| InternalError(e.to_string()))?
        .execute(sql.as_str(), &[])
        .map_err(|e| DBError(e.to_string()))?;
    Ok(())
}

pub fn table_all_clear() {
    //table_clear("accounts").unwrap();
    table_clear("users").unwrap();
    table_clear("coin_transaction").unwrap();
    //table_clear("wallet").unwrap();
    table_clear("secret_store").unwrap()
}
