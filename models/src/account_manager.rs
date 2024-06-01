extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::OpStatus;
use tokio_postgres::Row;
//use r2d2_postgres::postgres::{Row, Transaction};
use std::fmt;
use std::num::ParseIntError;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;

use crate::{vec_str2array_text, FilterContent, PgLocalCli, PsqlOp, PsqlType, UpdaterContent};
use anyhow::Result;

#[derive(Serialize, Debug)]
pub struct UserInfoEntity {
    pub id: u32,
    pub user_info: UserInfo,
    pub updated_at: String,
    pub created_at: String,
}

use serde::Serialize;
#[derive(Clone, Debug)]
pub enum UserFilter<'b> {
    ById(u32),
    ByPhone(&'b str),
    ByEmail(&'b str),
    ByPhoneOrEmail(&'b str),
    ByInviteCode(&'b str),
    ByAccountId(&'b str),
    ByMainAccount(&'b str),
}

#[derive(Clone, Debug)]
pub enum UserUpdater<'a> {
    LoginPwdHash(&'a str),
    AccountIds(Vec<String>),
    //     * anwser_indexes,secruity_is_seted,main_account
    SecruityInfo(&'a str, bool, &'a str),
    AnwserIndexes(&'a str),
    OpStatus(&'a str),
}

impl fmt::Display for UserUpdater<'_> {
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
            UserUpdater::OpStatus(status) => format!("op_status='{}'", status),
            UserUpdater::AnwserIndexes(anwser) => format!("anwser_indexes='{}' ", anwser),
        };
        write!(f, "{}", description)
    }
}

impl fmt::Display for UserFilter<'_> {
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
            UserFilter::ByMainAccount(id) => format!("main_account='{}' ", id),
        };
        write!(f, "{}", description)
    }
}

impl UserInfoEntity {
    pub fn new_with_specified(login_pwd_hash: &str) -> Self {
        let user = UserInfo {
            phone_number: "".to_string(),
            email: "".to_string(),
            login_pwd_hash: login_pwd_hash.to_owned(),
            anwser_indexes: "".to_string(),
            is_frozen: false,
            predecessor: None,
            laste_predecessor_replace_time: 0,
            invite_code: "".to_string(),
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
        UserInfoEntity {
            id: 0,
            user_info: user,
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

#[async_trait]
impl PsqlOp for UserInfoEntity {
    type UpdaterContent<'a> = UserUpdater<'a>;
    type FilterContent<'b> = UserFilter<'b>;
    async fn find(filter: Self::FilterContent<'_>, cli: &mut PgLocalCli<'_>) -> Result<Vec<Self>> {
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
            filter
        );
        let query_res = cli.query(&sql).await?;
        //debug!("get_snapshot: raw sql {}", sql);

        let gen_view = |row: &Row| -> Result<UserInfoEntity> {
            let view = UserInfoEntity {
                id: row.get::<usize, i32>(0) as u32,
                user_info: UserInfo {
                    phone_number: row.get(1),
                    email: row.get(2),
                    login_pwd_hash: row.get(3),
                    anwser_indexes: row.get(4),
                    is_frozen: row.get::<usize, bool>(5),
                    predecessor: row.get::<usize, Option<i32>>(6).map(|id| id as u32),
                    laste_predecessor_replace_time: row.get::<usize, String>(7).parse()?,
                    invite_code: row.get(8),
                    kyc_is_verified: row.get(9),
                    secruity_is_seted: row.get(10),
                    create_subacc_time: row
                        .get::<usize, Vec<String>>(11)
                        .iter()
                        .map(|t| {
                            t.parse()
                                .map_err(|e: ParseIntError| anyhow::anyhow!(e.to_string()))
                        })
                        .collect::<Result<Vec<u64>>>()?,
                    main_account: row.get(12),
                    op_status: row.get::<usize, String>(13).parse()?,
                    reserved_field1: row.get(14),
                    reserved_field2: row.get(15),
                    reserved_field3: row.get(16),
                },
                updated_at: row.get(17),
                created_at: row.get(18),
            };
            Ok(view)
        };
        query_res.iter().map(gen_view).collect()
    }

    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "UPDATE users SET {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update users {} ", sql);
        let execute_res = cli.execute(&sql).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update users {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(&self, cli: &mut PgLocalCli<'_>) -> Result<()> {
        let UserInfo {
            phone_number,
            email,
            login_pwd_hash,
            anwser_indexes,
            is_frozen,
            predecessor,
            laste_predecessor_replace_time,
            invite_code: _,
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
                kyc_is_verified,
                secruity_is_seted,
                create_subacc_time,
                main_account,
                op_status,
                reserved_field1,
                reserved_field2,
                reserved_field3
            ) values ('{}','{}','{}','{}',{},{},'{}',{},{},{},'{}','{}','{}','{}','{}')",
            phone_number,
            email,
            login_pwd_hash,
            anwser_indexes,
            is_frozen,
            PsqlType::OptionU64(predecessor.map(|x| x as u64)).to_psql_str(),
            laste_predecessor_replace_time,
            kyc_is_verified,
            secruity_is_seted,
            create_subacc_time_str,
            main_account,
            op_status,
            reserved_field1,
            reserved_field2,
            reserved_field3,
        );
        debug!("row sql {} rows", sql);
        let execute_res = cli.execute(&sql).await?;
        debug!("success insert {} rows", execute_res);
        Ok(())
    }
}

pub async fn get_next_uid(cli: &mut PgLocalCli<'_>) -> Result<u32> {
    let execute_res = cli
        .query("select last_value,is_called from users_id_seq order by last_value desc limit 1")
        .await?;
    //todo:
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

    use crate::general::{get_pg_pool_connect, transaction_begin, transaction_commit};

    use super::*;
    use common::log::init_logger;
    use std::env;
    use tokio_postgres::types::ToSql;

    #[tokio::test]
    async fn test_db_user_info() -> Result<()> {
        env::set_var("CONFIG", "/root/chainless_backend/config_test.toml");
        init_logger();
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

        let user = UserInfoEntity::new_with_specified("0123456789");
        user.insert(&mut db_cli).await.unwrap();
        let user_by_find = UserInfoEntity::find_single(UserFilter::ById(1), &mut db_cli)
            .await
            .unwrap();
        println!("{:?}", user_by_find);
        assert_eq!(user_by_find.user_info, user.user_info);
        UserInfoEntity::update(
            UserUpdater::LoginPwdHash("0123"),
            UserFilter::ById(1),
            &mut db_cli,
        )
        .await
        .unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn test_db_trans_user_info() {
        env::set_var("CONFIG", "/root/chainless_backend/config_test.toml");
        init_logger();
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();
        let mut db_cli = db_cli.begin().await.unwrap();

        let user = UserInfoEntity::new_with_specified("0123456789");
        user.insert(&mut db_cli).await.unwrap();
        let user_by_find = UserInfoEntity::find(UserFilter::ById(1), &mut db_cli)
            .await
            .unwrap();
        println!("by_conn2__{:?}", user_by_find);
        db_cli.commit().await.unwrap();

        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();
        let user_by_find = UserInfoEntity::find_single(UserFilter::ById(1), &mut db_cli)
            .await
            .unwrap();
        println!("by_trans3__{:?}", user_by_find);
        assert_eq!(user_by_find.user_info.login_pwd_hash, user.user_info.login_pwd_hash);
        UserInfoEntity::update(
            UserUpdater::LoginPwdHash("0123"),
            UserFilter::ById(1),
            &mut db_cli,
        )
        .await
        .unwrap();
    }
}
