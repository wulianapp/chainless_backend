extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::OpStatus;
use tokio_postgres::Row;
//use r2d2_postgres::postgres::{Row, Transaction};
use std::fmt;
use std::num::ParseIntError;
//#[derive(Serialize)]
use common::data_structures::account_manager::UserInfo;

use crate::{vec_str2array_text, FilterContent, PgLocalCli, PgLocalCli2, PsqlOp, PsqlType, UpdaterContent};
use anyhow::Result;

#[derive(Serialize, Debug)]
pub struct UserInfoEntity {
    pub user_info: UserInfo,
    pub updated_at: String,
    pub created_at: String,
}
impl UserInfoEntity {
    pub fn into_inner(self) -> UserInfo {
        self.user_info
    }
}

use serde::Serialize;
#[derive(Clone, Debug)]
pub enum UserFilter<'b> {
    ById(&'b u32),
    ByPhone(&'b str),
    ByEmail(&'b str),
    ByPhoneOrEmail(&'b str),
    ByInviteCode(&'b str),
    ByAccountId(&'b str),
    ByMainAccount(&'b str),
}

#[derive(Clone, Debug)]
pub enum UserUpdater<'a> {
    //pwd,token version
    LoginPwdHash(&'a str, u32),
    AccountIds(Vec<String>),
    //     * anwser_indexes,secruity_is_seted,main_account
    SecruityInfo(&'a str, &'a str),
    AnwserIndexes(&'a str),
    OpStatus(&'a str),
    Email(&'a str),
    PhoneNumber(&'a str),
    TokenVersion(u32),
}

impl fmt::Display for UserUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserUpdater::LoginPwdHash(pwd, version) => {
                format!("login_pwd_hash='{}',token_version={}", pwd, version)
            }
            UserUpdater::AccountIds(ids) => {
                let new_servant_str = super::vec_str2array_text(ids.to_owned());
                format!("account_ids={} ", new_servant_str)
            }
            UserUpdater::SecruityInfo(anwser_indexes, main_account) => format!(
                "anwser_indexes='{}',main_account='{}'",
                anwser_indexes, main_account
            ),
            UserUpdater::OpStatus(status) => format!("op_status='{}'", status),
            UserUpdater::AnwserIndexes(anwser) => format!("anwser_indexes='{}' ", anwser),
            UserUpdater::Email(email) => format!("email='{}'", email),
            UserUpdater::PhoneNumber(number) => format!("phone_number='{}'", number),
            UserUpdater::TokenVersion(version) => format!("token_version={}", version),
        };
        write!(f, "{}", description)
    }
}

impl fmt::Display for UserFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            UserFilter::ById(id) => format!("id={}", id),
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
    pub fn new_with_specified(user_id: u32, login_pwd_hash: &str) -> Self {
        let user = UserInfo {
            id: user_id,
            phone_number: None,
            email: None,
            login_pwd_hash: login_pwd_hash.to_owned(),
            anwser_indexes: "".to_string(),
            is_frozen: false,
            kyc_is_verified: false,
            create_subacc_time: vec![],
            main_account: None,
            token_version: 1,
        };
        UserInfoEntity {
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
            kyc_is_verified,\
            create_subacc_time,\
            main_account,\
            token_version,\
            cast(updated_at as text),\
            cast(created_at as text) \
            from users where {}",
            filter
        );
        //let query_res = PgLocalCli2::query(&sql).await?;
        let query_res = PgLocalCli2::query(&sql).await?;
        //debug!("get_snapshot: raw sql {}", sql);

        let gen_view = |row: &Row| -> Result<UserInfoEntity> {
            let view = UserInfoEntity {
                user_info: UserInfo {
                    id: row.get::<usize, i64>(0) as u32,
                    phone_number: row.get(1),
                    email: row.get(2),
                    login_pwd_hash: row.get(3),
                    anwser_indexes: row.get(4),
                    is_frozen: row.get::<usize, bool>(5),
                    kyc_is_verified: row.get(6),
                    create_subacc_time: row
                        .get::<usize, Vec<i64>>(7)
                        .into_iter()
                        .map(|t| t as u64)
                        .collect::<Vec<u64>>(),
                    main_account: row.get(8),
                    token_version: row.get::<usize, i64>(9) as u32,
                },
                updated_at: row.get(10),
                created_at: row.get(11),
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
        let execute_res = PgLocalCli2::execute(&sql).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update users {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self, cli: &mut PgLocalCli<'_>) -> Result<()> {
        let UserInfo {
            id,
            phone_number,
            email,
            login_pwd_hash,
            anwser_indexes,
            is_frozen,
            kyc_is_verified,
            create_subacc_time,
            main_account,
            token_version,
        } = self.into_inner();

        //assembly string array to sql string
        let create_subacc_time: PsqlType = create_subacc_time.into();
        let main_account: PsqlType = main_account.into();
        let phone_number: PsqlType = phone_number.into();
        let email: PsqlType = email.into();

        let sql = format!(
            "insert into users (\
                id,\
                phone_number,\
                email,\
                login_pwd_hash,\
                anwser_indexes,\
                is_frozen,\
                kyc_is_verified,\
                create_subacc_time,\
                main_account,\
                token_version\
            ) values ({},{},{},'{}','{}',{},{},{},{},{})",
            id,
            phone_number.to_psql_str(),
            email.to_psql_str(),
            login_pwd_hash,
            anwser_indexes,
            is_frozen,
            kyc_is_verified,
            create_subacc_time.to_psql_str(),
            main_account.to_psql_str(),
            token_version
        );
        debug!("row sql {} rows", sql);
        let execute_res = PgLocalCli2::execute(&sql).await?;
        debug!("success insert {} rows", execute_res);
        Ok(())
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

        let user = UserInfoEntity::new_with_specified(123245, "0123456789");
        user.insert(&mut db_cli).await.unwrap();
        let user_by_find = UserInfoEntity::find_single(UserFilter::ById(&123245), &mut db_cli)
            .await
            .unwrap();
        println!("{:?}", user_by_find);
        //assert_eq!(user_by_find.user_info, user.user_info);
        UserInfoEntity::update(
            UserUpdater::LoginPwdHash("0123", 2),
            UserFilter::ById(&1),
            &mut db_cli,
        )
        .await
        .unwrap();
        Ok(())
    }

    /*** 
    #[tokio::test]
    async fn test_db_trans_user_info() {
        env::set_var("CONFIG", "/root/chainless_backend/config_test.toml");
        init_logger();
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli2 = get_pg_pool_connect5().await.unwrap();
        let mut db_cli = db_cli.begin().await.unwrap();

        let user = UserInfoEntity::new_with_specified(12345, "0123456789");
        user.insert(&mut db_cli).await.unwrap();
        let user_by_find = UserInfoEntity::find(UserFilter::ById(&1), &mut db_cli)
            .await
            .unwrap();
        println!("by_conn2__{:?}", user_by_find);
        db_cli.commit().await.unwrap();

        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();
        let user_by_find = UserInfoEntity::find_single(UserFilter::ById(&12345), &mut db_cli)
            .await
            .unwrap();
        println!("by_trans3__{:?}", user_by_find);
        //assert_eq!(user_by_find.user_info.login_pwd_hash, user.user_info.login_pwd_hash);
        UserInfoEntity::update(
            UserUpdater::LoginPwdHash("0123", 2),
            UserFilter::ById(&1),
            &mut db_cli,
        )
        .await
        .unwrap();
    }
    ***/
}
