extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::airdrop::{Airdrop, BtcGradeStatus};

use serde::{Deserialize, Serialize};

use std::fmt;

use tokio_postgres::Row;

use crate::{PgLocalCli, PsqlOp, PsqlType};
use anyhow::{Ok, Result};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct AirdropEntity {
    pub airdrop: Airdrop,
    pub updated_at: String,
    pub created_at: String,
}

impl AirdropEntity {
    pub fn into_inner(self) -> Airdrop {
        self.airdrop
    }
}

#[derive(Debug)]
pub enum AirdropUpdater<'a> {
    InviteCode(&'a str),
    BtcAddress(&'a str),
    BtcAddressAndLevel(&'a str, Option<u8>),
    AccountId(&'a str),
    //user_id,account_id
    Predecessor(&'a u32, &'a str),
    BtcLevel(u8),
    GradeStatus(BtcGradeStatus),
    LevelStatus(u8,BtcGradeStatus),
    BtcAddrLevelStatus(Option<String>,u8,BtcGradeStatus),
    ResetBind,
}

impl fmt::Display for AirdropUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AirdropUpdater::ResetBind => {
                format!("ref_btc_address=NULL,btc_address=NULL,btc_level=0,btc_grade_status='NotBind' ")
            }
            AirdropUpdater::BtcAddrLevelStatus(addr,level,status) => {
                let addr: PsqlType = addr.to_owned().into();
                format!("btc_address={},btc_level='{}',btc_grade_status='{}' ", addr.to_psql_str(),level,status.to_string())
            }
            AirdropUpdater::InviteCode(code) => {
                format!("invite_code='{}'", code)
            }
            AirdropUpdater::BtcAddress(addr) => {
                format!("btc_address='{}'", addr)
            }
            AirdropUpdater::BtcAddressAndLevel(addr, level) => {
                let have_grade = level.is_none();
                let level: PsqlType = level.to_owned().into();
                if have_grade {
                    format!("btc_address='{}',btc_grade_status='PendingCalculate',btc_level={} ", addr, level.to_psql_str())
                }else{
                    format!("btc_address='{}',btc_grade_status='Calculated',btc_level={} ", addr, level.to_psql_str())
                }
                
            }
            AirdropUpdater::LevelStatus(level,status) => {
                format!("btc_level='{}',btc_grade_status='{}' ", level,status.to_string())
            }
            AirdropUpdater::AccountId(id) => {
                format!("account_id='{}'", id)
            }
            AirdropUpdater::Predecessor(user_id, account_id) => {
                format!(
                    "predecessor_user_id={},predecessor_account_id='{}'",
                    user_id, account_id
                )
            }
            AirdropUpdater::BtcLevel(level) => {
                format!("btc_level={}", level)
            }
            AirdropUpdater::GradeStatus(status) => {
                format!("btc_grade_status='{}'", status.to_string())
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum AirdropFilter<'b> {
    ByInviteCode(&'b str),
    ByAccountId(&'b str),
    ByBtcAddress(&'b str),
    ByUserId(&'b u32),
}

impl fmt::Display for AirdropFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            AirdropFilter::ByInviteCode(code) => format!("invite_code='{}' ", code),
            AirdropFilter::ByAccountId(id) => format!("account_id='{}' ", id),
            AirdropFilter::ByUserId(id) => format!("user_id={} ", id),
            AirdropFilter::ByBtcAddress(addr) => format!("btc_address='{}' ", addr),
        };
        write!(f, "{}", description)
    }
}

impl AirdropEntity {
    pub fn new_with_specified(
        user_id: u32,
        predecessor_user_id: u32,
        predecessor_account_id: &str,
    ) -> Self {
        AirdropEntity {
            airdrop: Airdrop {
                user_id,
                account_id: None,
                invite_code: user_id.to_string(),
                predecessor_user_id,
                predecessor_account_id: predecessor_account_id.to_string(),
                btc_address: None,
                btc_level: None,
                btc_grade_status: BtcGradeStatus::NotBind,
                ref_btc_address: None,
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}
#[async_trait]
impl PsqlOp for AirdropEntity {
    type UpdaterContent<'a> = AirdropUpdater<'a>;
    type FilterContent<'b> = AirdropFilter<'b>;
    async fn find(filter: Self::FilterContent<'_>) -> Result<Vec<AirdropEntity>> {
        let sql = format!(
            "select 
            user_id,\
            account_id,\
            invite_code,\
            predecessor_user_id,\
            predecessor_account_id,\
            btc_address,\
            btc_level,\
            btc_grade_status,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from airdrop where {}",
            filter
        );
        let execute_res = PgLocalCli::query(sql.as_str()).await?;
        debug!("get_airdrop: raw sql {}", sql);
        let gen_view = |row: &Row| -> Result<AirdropEntity>{
            Ok(AirdropEntity {
                airdrop: Airdrop {
                    user_id: row.get::<usize, i64>(0) as u32,
                    account_id: row.get::<usize, Option<String>>(1),
                    invite_code: row.get(2),
                    predecessor_user_id: row.get::<usize, i64>(3) as u32,
                    predecessor_account_id: row.get::<usize, String>(4),
                    btc_address: row.get::<usize, Option<String>>(5),
                    btc_level: row.get::<usize, Option<i16>>(6).map(|x| x as u8),
                    btc_grade_status: row.get::<usize, String>(7).parse()?,
                    ref_btc_address: row.get::<usize, Option<String>>(8),
                },
                updated_at: row.get(9),
                created_at: row.get(10),
            })
        };

        execute_res.iter().map(gen_view).collect()
    }
    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update airdrop set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = PgLocalCli::execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self) -> Result<()> {
        let Airdrop {
            user_id,
            account_id,
            invite_code,
            predecessor_user_id,
            predecessor_account_id,
            btc_address,
            btc_level,
            btc_grade_status,
            ref_btc_address
        } = self.into_inner();
        let account_id: PsqlType = account_id.into();
        let btc_address: PsqlType = btc_address.into();
        let btc_level: PsqlType = btc_level.into();
        let ref_btc_address: PsqlType = ref_btc_address.into();


        let sql = format!(
            "insert into airdrop (\
                user_id,\
                account_id,\
                invite_code,\
                predecessor_user_id,\
                predecessor_account_id,\
                btc_address,\
                btc_level,\
                btc_grade_status,\
                ref_btc_address
         ) values ('{}',{},'{}',{},'{}',{},{},'{}',{});",
            user_id,
            account_id.to_psql_str(),
            invite_code,
            predecessor_user_id,
            predecessor_account_id,
            btc_address.to_psql_str(),
            btc_level.to_psql_str(),
            btc_grade_status.to_string(),
            ref_btc_address.to_psql_str()
        );
        debug!("row sql {} rows", sql);
        let _execute_res = PgLocalCli::execute(sql.as_str()).await?;
        Ok(())
    }

    async fn delete(_filter: Self::FilterContent<'_>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::general::{run_api_call, table_clear};

    use super::*;
    use common::log::init_logger;
    use std::env;

    #[tokio::test]
    async fn test_db_airdop() {
        env::set_var("CONFIG", "/root/chainless_backend/config_test.toml");
        init_logger();
        table_clear("airdrop").await.unwrap();
        let task = async {
            let airdrop = AirdropEntity::new_with_specified(1, 2, "3.local");
            airdrop.insert().await.unwrap();
            let airdrop_by_find = AirdropEntity::find_single(AirdropFilter::ByInviteCode("1"))
                .await
                .unwrap();
            println!("{:?}", airdrop_by_find);
            //assert_eq!(airdrop.airdrop, airdrop_by_find.airdrop);

            AirdropEntity::update_single(
                AirdropUpdater::InviteCode("3"),
                AirdropFilter::ByInviteCode("1"),
            )
            .await
            .unwrap();
        };
        run_api_call("", task).await.unwrap();
    }
}
