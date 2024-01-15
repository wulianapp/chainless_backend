extern crate rustc_serialize;

use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;
use common::utils::time::get_current_time;
use serde::Serialize;
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
            UserFilter::ByPhoneOrEmail(contact) => {
                format!("email='{}' or phone_number='{}'", contact, contact)
            }
        };
        filter_str
    }
}

pub fn get_current_user_num() -> u64 {
    let execute_res = crate::query("select count(1) from users").unwrap();
    let user_info_raw = execute_res.first().unwrap();
    let user_num = user_info_raw.get::<usize, i64>(0) as u64;
    user_num
}

//取当前和一天之前的快照
pub fn get_by_user(filter: UserFilter) -> Option<UserInfoView> {
    let sql = format!(
        "select id,\
         phone_number,email,state,\
         multi_sign_strategy,verified,pwd_hash,invite_code,\
         direct_invited_number,ancestors,points,grade,fans_num, \
         cast(updated_at as text), cast(created_at as text) \
         from users where {}",
        filter.to_string()
    );
    let execute_res = crate::query(sql.as_str()).unwrap();
    info!("get_snapshot: raw sql {}", sql);
    if execute_res.is_empty() {
        return None;
    }
    if execute_res.len() > 1 {
        //todo:throw error
        //return None
        panic!("_tmp");
    }
    let user_info_raw = execute_res.first().unwrap();
    let gen_snapshot = |row: &Row| UserInfoView {
        id: row.get::<usize, i32>(0) as u32,
        user_info: UserInfo {
            phone_number: row.get(1),
            email: row.get(2),
            state: row.get::<usize, i16>(3) as u8,
            multi_sign_strategy: row.get(4),
            verified: row.get(5),
            pwd_hash: row.get(6),
            invite_code: row.get(7),
            direct_invited_number: row.get::<usize, i32>(8) as u32,
            ancestors: row.get::<usize, Vec<String>>(9),
            points: row.get::<usize, i32>(10) as u32,
            grade: row.get::<usize, i16>(11) as u8,
            fans_num: row.get::<usize, i32>(12) as u32,
        },
        updated_at: row.get(13),
        created_at: row.get(14),
    };
    Some(gen_snapshot(user_info_raw))
}

pub fn single_insert(data: UserInfo) -> Result<(), String> {
    let now = get_current_time();
    let UserInfo {
        phone_number,
        email,
        state,
        multi_sign_strategy,
        verified,
        pwd_hash,
        invite_code,
        direct_invited_number,
        ancestors: _,
        points,
        grade,
        fans_num,
    } = data;
    //todo: convert ancestors text array
    let sql = format!("insert into users (phone_number,email,state,\
    multi_sign_strategy,verified,pwd_hash,invite_code,direct_invited_number,\
    ancestors,points,grade,fans_num,updated_at) values ('{}','{}',{},'{}','{}','{}','{}','{}',ARRAY[]::text[],{},{},{},'{}');",
                          phone_number,email,state, multi_sign_strategy,verified,pwd_hash,
                          invite_code,direct_invited_number, /***ancestors,*/points,grade,fans_num,now
    );
    println!("row sql {} rows", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert {} rows", execute_res);
    Ok(())
}

pub fn update_password(new_password: &str, filter: UserFilter) {
    let sql = format!(
        "UPDATE users SET pwd_hash='{}' where {}",
        new_password,
        filter.to_string()
    );
    info!("start update orders {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update orders {} rows", execute_res);
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
        user.invite_code = format!("{}", invite_code);
        println!("start insert");
        single_insert(user.clone()).unwrap();
        println!("start query");
        let res = get_by_user(UserFilter::ByEmail(&user.email));
        println!("res {:?}", res.unwrap());
    }
}
