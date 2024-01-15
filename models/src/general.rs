pub fn transaction_begin() {
    let _res = crate::CLIENTDB
        .lock()
        .unwrap()
        .simple_query("BEGIN")
        .unwrap();
}

pub fn transaction_commit() {
    crate::CLIENTDB
        .lock()
        .unwrap()
        .simple_query("commit")
        .unwrap();
}

//client.execute(
//         "DROP TABLE users",
//         &[],
//     )?;
pub fn table_clear(table_name: &str) {
    let sql = format!("truncate table {} restart identity", table_name);
    crate::CLIENTDB
        .lock()
        .unwrap()
        .execute(sql.as_str(), &[])
        .unwrap();
}

pub fn table_all_clear() {
    table_clear("accounts");
    table_clear("users");
    table_clear("coin_transaction");
    table_clear("wallet")
}
