extern crate rustc_serialize;

use std::fmt::format;
use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;
use common::utils::time::current_date;
use serde::Serialize;
use common::error_code::BackendError;
use crate::vec_str2array_text;

#[derive(Serialize, Debug, Default)]
pub struct UserInfoView {
    pub id: u32,
    pub user_info: UserInfo,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub enum UserFilter<'a> {
    ById(u32),
    ByPhone(&'a str),
    //market_id
    ByEmail(&'a str),
    ByPhoneOrEmail(&'a str),
    ByInviteCode(&'a str),
    ByAccountId(&'a str),
}

impl UserFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            UserFilter::ById(id) => {
                format!("id='{}'", id)
            }
            UserFilter::ByPhone(index) => {
                format!("phone_number='{}'", index)
            }
            UserFilter::ByEmail(email) => {
                format!("email='{}'", email)
            }
            UserFilter::ByInviteCode(code) => {
                format!("invite_code='{}'", code)
            }
            UserFilter::ByPhoneOrEmail(contact) => {
                format!("email='{}' or phone_number='{}'", contact, contact)
            }
            UserFilter::ByAccountId(id) => {
                format!("'{}'=any(account_ids) ", id)
            }
        };
        filter_str
    }
}

pub fn get_current_user_num() -> Result<u64,BackendError> {
    let execute_res = crate::query("select count(1) from users")?;
    let user_info_raw = execute_res.first().unwrap();
    let user_num = user_info_raw.get::<usize, i64>(0) as u64;
    Ok(user_num)
}

pub fn get_next_uid() -> Result<u32,BackendError> {
    let execute_res = crate::query("select last_value from users_id_seq order by last_value desc limit 1")?;
    if let Some(row) = execute_res.first() {
        let current_user_id = row.get::<usize, i64>(0) as u32;
        Ok(current_user_id + 1)
    }else {
        Ok(1)
    }
}

//取当前和一天之前的快照
pub fn get_user(filter: UserFilter) -> Result<Option<UserInfoView>,BackendError>{
    let sql = format!(
        "select id,phone_number,email,\
         pwd_hash,predecessor,status,verified,invite_code,account_ids,\
         cast(updated_at as text), cast(created_at as text) \
         from users where {}",
        filter.to_string()
    );
    let execute_res = crate::query(sql.as_str())?;
    debug!("get_snapshot: raw sql {}", sql);
    if execute_res.is_empty() {
        return Ok(None);
    }

    //fixme:
    let user_info_raw = execute_res.first().unwrap();
    let gen_snapshot = |row: &Row| {
        UserInfoView {
            id: row.get::<usize, i32>(0) as u32,
            user_info: UserInfo {
                phone_number: row.get(1),
                email: row.get(2),
                pwd_hash: row.get(3),
                predecessor:  row.get::<usize, Option<i32>>(4).map(|id| id as u32),
                status:  row.get::<usize, i16>(5) as u8,
                verified:  row.get(6),
                invite_code: row.get(7),
                account_ids: row.get::<usize, Vec<String>>(8),
            },
            updated_at: row.get(9),
            created_at: row.get(10),
        }
    };
    Ok(Some(gen_snapshot(user_info_raw)))
}

pub fn single_insert(data: &UserInfo) -> Result<(), BackendError> {
    let UserInfo {
        phone_number,
        email,
        pwd_hash,
        predecessor,
        verified,
        status,
        invite_code,
        account_ids,
    } = data;

    let predecessor_str = predecessor.map(|x| format!("{}",x)).unwrap_or("NULL".to_string());
    //assembly string array to sql string
    let account_ids_str = vec_str2array_text(account_ids.to_owned());

    let sql = format!("insert into users (phone_number,email,pwd_hash,\
    predecessor,verified,status,invite_code,account_ids) values ('{}','{}','{}',{},{},{},{},{});",
                      phone_number,email,pwd_hash,predecessor_str,verified,status,invite_code,account_ids_str
    );
    debug!("row sql {} rows", sql);
    let execute_res = crate::execute(sql.as_str())?;
    debug!("success insert {} rows", execute_res);
    Ok(())
}

pub fn update_password(new_password: &str, filter: UserFilter) -> Result<(),BackendError>{
    let sql = format!(
        "UPDATE users SET pwd_hash='{}' where {}",
        new_password,
        filter.to_string()
    );
    info!("start update orders {} ", sql);
    let execute_res = crate::execute(sql.as_str())?;
    info!("success update orders {} rows", execute_res);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::utils::math;
    #[test]
    fn test_account_manager_braced() {
        let invite_code = math::gen_random_verify_code();
        let mut user = UserInfo::default();
        user.email = format!("example_{}@gmail.com", invite_code);
        user.predecessor = format!("{}", invite_code);
        println!("start insert");
        single_insert(user.clone()).unwrap();
        println!("start query");
        let res = get_user(UserFilter::ByEmail(&user.email));
        println!("res {:?}", res.unwrap());
    }
}
