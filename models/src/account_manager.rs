extern crate rustc_serialize;

use common::data_structures::OpStatus;
use postgres::Row;
use std::fmt;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;

use crate::coin_transfer::{CoinTxFilter, CoinTxView};
use crate::{vec_str2array_text, PsqlOp, PsqlType};
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType};
use common::error_code::BackendError;
use serde::Serialize;

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
    //     * anwser_indexes,secruity_is_seted,main_account
    SecruityInfo(String, bool, String),
    OpStatus(OpStatus),
}
impl fmt::Display for UserUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserUpdater::LoginPwdHash(pwd) => format!("login_pwd_hash='{}'", pwd),
            UserUpdater::AccountIds(ids) => {
                let new_servant_str = super::vec_str2array_text(ids.to_owned());
                format!("account_ids={} ", new_servant_str)
            }
            UserUpdater::SecruityInfo(anwser_indexes, secruity_is_seted, main_account) => format!(
                "anwser_indexes='{}',secruity_is_seted={},main_account='{}'",
                anwser_indexes, secruity_is_seted, main_account
            ),
            UserUpdater::OpStatus(status) => format!(
                "op_status='{}'",status.to_string()
            ),
        };
        write!(f, "{}", description)
    }
}

impl fmt::Display for UserFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserFilter::ById(id) => format!("id='{}'", id),
            UserFilter::ByPhone(number) => format!("phone_number='{}'", number),
            UserFilter::ByEmail(email) => format!("email='{}'", email),
            UserFilter::ByInviteCode(code) => format!("invite_code='{}'", code),
            UserFilter::ByPhoneOrEmail(contact) => {
                format!("email='{}' or phone_number='{}'", contact, contact)
            }
            UserFilter::ByAccountId(id) => format!("'{}'=any(account_ids) ", id),
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
    pub fn new_with_specified(login_pwd_hash: &str, invite_code: &str) -> Self {
        let user = UserInfo {
            phone_number: "".to_string(),
            email: "".to_string(),
            login_pwd_hash: login_pwd_hash.to_owned(),
            anwser_indexes: "".to_string(),
            is_frozen: false,
            predecessor: None,
            laste_predecessor_replace_time: 0,
            invite_code: invite_code.to_owned(),
            kyc_is_verified: false,
            secruity_is_seted: false,
            create_subacc_time: vec![],
            //fixme: replace with None
            main_account: "".to_string(),
            op_status: OpStatus::Idle,
            reserved_field1: "".to_string(),
            reserved_field2: "".to_string(),
            reserved_field3: "".to_string(),

        };
        UserInfoView {
            id: 0,
            user_info: user,
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

impl PsqlOp for UserInfoView {
    type UpdateContent = UserUpdater;
    type FilterContent = UserFilter;

    fn find(filter: Self::FilterContent) -> Result<Vec<Self>, BackendError> {
        let sql = format!(
            "select id,\
            phone_number,\
            email,\
            login_pwd_hash,\
            anwser_indexes,\
            is_frozen,\
            predecessor,\
            laste_predecessor_replace_time,\
            invite_code,\
            kyc_is_verified,\
            secruity_is_seted,\
            create_subacc_time,\
            main_account,\
            op_status,\
            reserved_field1,\
            reserved_field2,\
            reserved_field3,\
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
                anwser_indexes: row.get(4),
                is_frozen: row.get::<usize, bool>(5),
                predecessor: row.get::<usize, Option<i32>>(6).map(|id| id as u32),
                laste_predecessor_replace_time: row.get::<usize, String>(7).parse().unwrap(),
                invite_code: row.get(8),
                kyc_is_verified: row.get(9),
                secruity_is_seted: row.get(10),
                create_subacc_time: row
                    .get::<usize, Vec<String>>(11)
                    .iter()
                    .map(|t| t.parse().unwrap())
                    .collect(),
                main_account: row.get(12),
                op_status: row.get::<usize, String>(13).parse().unwrap(),
                reserved_field1: row.get(14),
                reserved_field2: row.get(15),
                reserved_field3: row.get(16),
            },
            updated_at: row.get(17),
            created_at: row.get(18),
        };
        let users = query_res.iter().map(|x| gen_view(x)).collect();
        Ok(users)
    }

    fn update(
        new_value: Self::UpdateContent,
        filter: Self::FilterContent,
    ) -> Result<(), BackendError> {
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
            anwser_indexes,
            is_frozen,
            predecessor,
            laste_predecessor_replace_time,
            invite_code,
            kyc_is_verified,
            secruity_is_seted,
            create_subacc_time,
            main_account,
            op_status,
            reserved_field1,
            reserved_field2,
            reserved_field3,
        } = &self.user_info;

        let _predecessor_str = predecessor
            .map(|x| format!("{}", x))
            .unwrap_or("NULL".to_string());
        //assembly string array to sql string
        let create_subacc_time = create_subacc_time.iter().map(|x| x.to_string()).collect();
        let create_subacc_time_str = vec_str2array_text(create_subacc_time);

        let sql = format!(
            "insert into users (phone_number,
                email,
                login_pwd_hash,\
                anwser_indexes,
                is_frozen,
                predecessor,
                laste_predecessor_replace_time,
                invite_code,
                kyc_is_verified,
                secruity_is_seted,
                create_subacc_time,
                main_account,
                op_status,
                reserved_field1,
                reserved_field2,
                reserved_field3
            ) values ('{}','{}','{}','{}',{},{},'{}','{}',{},{},{},'{}','{}','{}','{}','{}');",
            phone_number,
            email,
            login_pwd_hash,
            anwser_indexes,
            is_frozen,
            PsqlType::OptionU64(predecessor.map(|x| x as u64)).to_psql_str(),
            laste_predecessor_replace_time,
            invite_code,
            kyc_is_verified,
            secruity_is_seted,
            create_subacc_time_str,
            main_account,
            op_status.to_string(),
            reserved_field1,
            reserved_field2,
            reserved_field3,
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

    use super::*;
    use common::log::init_logger;
    use postgres::types::ToSql;
    use std::env;

    #[test]
    fn test_db_user_info() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let user = UserInfoView::new_with_specified("0123456789", "1");
        user.insert().unwrap();
        let user_by_find = UserInfoView::find_single(UserFilter::ById(1)).unwrap();
        println!("{:?}", user_by_find);
        assert_eq!(user_by_find.user_info, user.user_info);

        UserInfoView::update(
            UserUpdater::LoginPwdHash("0123".to_string()),
            UserFilter::ById(1),
        )
        .unwrap();
    }
}
