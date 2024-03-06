extern crate rustc_serialize;

use std::fmt;
use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;

use crate::{vec_str2array_text, PsqlOp, PsqlType};
use common::error_code::BackendError;
use serde::Serialize;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType};
use crate::coin_transfer::{CoinTxFilter, CoinTxView};

#[derive(Clone, Debug)]
pub enum UserFilter {
    ById(u32),
    ByPhone(String),
    ByEmail(String),
    ByPhoneOrEmail(String),
    ByInviteCode(String),
    ByAccountId(String),
}

#[derive(Clone, Debug)]
pub enum UserUpdater {
    LoginPwdHash(String),
    AccountIds(Vec<String>),
    //     * sign_pwd_hash,secruity_is_seted,main_account
    SecruityInfo(String,bool,String)
}
impl fmt::Display for UserUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserUpdater::LoginPwdHash(pwd) =>  format!("login_pwd_hash='{}'", pwd),
            UserUpdater::AccountIds(ids) =>  {
                let new_servant_str = super::vec_str2array_text(ids.to_owned());
                format!("account_ids={} ", new_servant_str)
            },
            UserUpdater::SecruityInfo(sign_pwd_hash,secruity_is_seted,main_account) => 
             format!("sign_pwd_hash='{}',secruity_is_seted={},main_account='{}'",
             sign_pwd_hash,secruity_is_seted,main_account
            ),
        };
        write!(f, "{}", description)
    }
}

impl fmt::Display for UserFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserFilter::ById(id) =>  format!("id='{}'", id),
            UserFilter::ByPhone(number) =>    format!("phone_number='{}'", number),
            UserFilter::ByEmail(email) =>  format!("email='{}'", email),
            UserFilter::ByInviteCode(code) =>   format!("invite_code='{}'", code),
            UserFilter::ByPhoneOrEmail(contact) =>  format!("email='{}' or phone_number='{}'", contact, contact),
            UserFilter::ByAccountId(id) =>   format!("'{}'=any(account_ids) ", id),
        };
        write!(f, "{}", description)
    }
}

#[derive(Serialize, Debug)]
pub struct UserInfoView {
    pub id: u32,
    pub user_info: UserInfo,
    pub updated_at: String,
    pub created_at: String,
}

impl UserInfoView {
    pub fn new_with_specified(
        login_pwd_hash:&str,
        invite_code:&str,
    ) -> Self{
        let user = UserInfo{
            phone_number: "".to_string(),
            email: "".to_string(),
            login_pwd_hash:login_pwd_hash.to_owned(),
            sign_pwd_hash: "".to_string(),
            is_frozen: false,
            predecessor: None,
            laste_predecessor_replace_time: 0,
            invite_code: invite_code.to_owned(),
            kyc_is_verified: false,
            secruity_is_seted: false,
            create_subacc_time: vec![],
            //fixme: replace with None
            main_account: "".to_string(),
        };
        UserInfoView{
            id: 0,
            user_info: user,
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

impl PsqlOp for UserInfoView{
    type UpdateContent = UserUpdater;
    type FilterContent = UserFilter;

    fn find(filter: Self::FilterContent) -> Result<Vec<Self>, BackendError> {
        let sql = format!(
            "select id,\
            phone_number,\
            email,\
            login_pwd_hash,\
            sign_pwd_hash,\
            is_frozen,\
            predecessor,\
            laste_predecessor_replace_time,\
            invite_code,\
            kyc_is_verified,\
            secruity_is_seted,\
            create_subacc_time,\
            main_account,\
            cast(updated_at as text),\
            cast(created_at as text) \
            from users where {}",
            filter.to_string()
        );
        let query_res = crate::query(sql.as_str())?;
        debug!("get_snapshot: raw sql {}", sql);

        let gen_view = |row: &Row| UserInfoView {
            id: row.get::<usize, i32>(0) as u32,
            user_info: UserInfo {
                phone_number: row.get(1),
                email: row.get(2),
                login_pwd_hash: row.get(3),
                sign_pwd_hash: row.get(4),
                is_frozen: row.get::<usize, bool>(5),
                predecessor: row.get::<usize, Option<i32>>(6).map(|id| id as u32),
                laste_predecessor_replace_time: row.get::<usize, String>(7).parse().unwrap(),
                invite_code: row.get(8),
                kyc_is_verified: row.get(9),
                secruity_is_seted: row.get(10),
                create_subacc_time: row.get::<usize, Vec<String>>(11).iter().map(|t| t.parse().unwrap()).collect(),
                main_account: row.get(12),
            },
            updated_at: row.get(13),
            created_at: row.get(14),
        };
        let users = query_res.iter().map(|x| gen_view(x)).collect();
        Ok(users)
    }

    fn update(new_value: Self::UpdateContent, filter: Self::FilterContent) -> Result<(), BackendError> {
        let sql = format!(
            "UPDATE users SET {} where {}",
            new_value.to_string(),
            filter.to_string()
        );
        debug!("start update users {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        debug!("success update users {} rows", execute_res);
        Ok(())
    }
    

    fn insert(&self) -> Result<(), BackendError> {
        let UserInfo {
            phone_number,
            email,
            login_pwd_hash,
            sign_pwd_hash,
            is_frozen,
            predecessor,
            laste_predecessor_replace_time,
            invite_code,
            kyc_is_verified,
            secruity_is_seted,
            create_subacc_time,
            main_account,
        } = &self.user_info;

        let predecessor_str = predecessor
            .map(|x| format!("{}", x))
            .unwrap_or("NULL".to_string());
        //assembly string array to sql string
        let create_subacc_time = create_subacc_time
        .iter().map(|x| x.to_string()).collect();
        let create_subacc_time_str = vec_str2array_text(create_subacc_time);

        let sql = format!(
            "insert into users (phone_number,
                email,
                login_pwd_hash,\
                sign_pwd_hash,
                is_frozen,
                predecessor,
                laste_predecessor_replace_time,
                invite_code,
                kyc_is_verified,
                secruity_is_seted,
                create_subacc_time,
                main_account
            ) values ('{}','{}','{}','{}',{},{},'{}','{}',{},{},{},'{}');",
            phone_number,
            email,
            login_pwd_hash,
            sign_pwd_hash,
            is_frozen,
            PsqlType::OptionU64(predecessor.map(|x| x as u64)).to_psql_str(),
            laste_predecessor_replace_time,
            invite_code,
            kyc_is_verified,
            secruity_is_seted,
            create_subacc_time_str,
            main_account
        );
        debug!("row sql {} rows", sql);
        let execute_res = crate::execute(sql.as_str())?;
        debug!("success insert {} rows", execute_res);
        Ok(())
    }
}



pub fn get_current_user_num() -> Result<u64, BackendError> {
    let execute_res = crate::query("select count(1) from users")?;
    let user_info_raw = execute_res.first().unwrap();
    let user_num = user_info_raw.get::<usize, i64>(0) as u64;
    Ok(user_num)
}

pub fn get_next_uid() -> Result<u32, BackendError> {
    let execute_res = crate::query(
        "select last_value,is_called from users_id_seq order by last_value desc limit 1",
    )?;
    let row = execute_res.first().unwrap();
    let current_user_id = row.get::<usize, i64>(0) as u32;
    let is_called = row.get::<usize, bool>(1);
    //auto index is always 1 when no user or insert one
    if is_called {
        Ok(current_user_id + 1)
    } else {
        Ok(1)
    }
}

#[cfg(test)]
mod tests {

    use std::env;
    use super::*;
    use common::log::init_logger;
    use postgres::types::ToSql;

    #[test]
    fn test_db_user_info() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let user = UserInfoView::new_with_specified(
            "0123456789", "1");
            user.insert().unwrap();
        let mut user_by_find = UserInfoView::find_single(
            UserFilter::ById(1)).unwrap();
        println!("{:?}",user_by_find);
        assert_eq!(user_by_find.user_info,user.user_info);   

        UserInfoView::update(
            UserUpdater::LoginPwdHash("0123".to_string()), 
            UserFilter::ById(1), 
        ).unwrap();
    }
}
