extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::device_info::DeviceInfo;
use std::fmt;

use tokio_postgres::Row;
//#[derive(Serialize)]

use common::data_structures::SecretKeyState;

use serde::{Deserialize, Serialize};

use crate::{PgLocalCli, PsqlOp, PsqlType};
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceInfoEntity {
    pub device_info: DeviceInfo,
    pub updated_at: String,
    pub created_at: String,
}

impl DeviceInfoEntity {
    pub fn into_inner(self) -> DeviceInfo {
        self.device_info
    }
}

#[derive(Debug)]
pub enum DeviceInfoUpdater<'a> {
    State(SecretKeyState),
    HolderSaved(bool),
    BecomeMaster(&'a str),
    BecomeServant(&'a str),
    AddServant(&'a str),
    BecomeUndefined(&'a str),
}

impl fmt::Display for DeviceInfoUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoUpdater::State(new_state) => {
                format!("state='{}'", new_state)
            }
            DeviceInfoUpdater::HolderSaved(saved) => {
                format!("holder_confirm_saved={} ", saved)
            }
            DeviceInfoUpdater::BecomeMaster(key) => {
                format!(
                    "(hold_pubkey,holder_confirm_saved,key_role)=('{}',true,'Master') ",
                    key
                )
            }
            DeviceInfoUpdater::BecomeServant(key) => {
                format!(
                    "(hold_pubkey,holder_confirm_saved,key_role)=('{}','true','Servant') ",
                    key
                )
            }
            DeviceInfoUpdater::AddServant(key) => {
                format!("(hold_pubkey,key_role)=('{}','Servant') ", key)
            }
            DeviceInfoUpdater::BecomeUndefined(key) => {
                format!(
                    "(hold_pubkey,holder_confirm_saved,key_role)=('{}',true,'Undefined') ",
                    key
                )
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum DeviceInfoFilter<'b> {
    ByUser(&'b u32),
    /// device_id,user_id
    ByDeviceUser(&'b str, &'b u32),
    ByUserDeviceHoldSecret(u32, &'b str, bool),
    ByHoldKey(&'b str),
}

impl fmt::Display for DeviceInfoFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoFilter::ByUser(user_id) => format!("user_id={} order by created_at", user_id),
            DeviceInfoFilter::ByDeviceUser(device_id, user_id) => {
                format!("id='{}' and user_id={} ", device_id, user_id)
            }
            DeviceInfoFilter::ByUserDeviceHoldSecret(user_id, device_id, saved) => format!(
                "user_id={} and id='{}' and holder_confirm_saved={} ",
                user_id, device_id, saved
            ),
            DeviceInfoFilter::ByHoldKey(key) => {
                format!("hold_pubkey='{}' ", key)
            }
        };
        write!(f, "{}", description)
    }
}

impl DeviceInfoEntity {
    pub fn new_with_specified(id: &str, brand: &str, user_id: u32) -> Self {
        DeviceInfoEntity {
            device_info: DeviceInfo {
                id: id.to_owned(),
                user_id,
                state: common::data_structures::DeviceState::Active,
                hold_pubkey: None,
                brand: brand.to_owned(),
                holder_confirm_saved: false,
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

#[async_trait]
impl PsqlOp for DeviceInfoEntity {
    type UpdaterContent<'a> = DeviceInfoUpdater<'a>;
    type FilterContent<'b> = DeviceInfoFilter<'b>;

    async fn find(filter: Self::FilterContent<'_>) -> Result<Vec<Self>> {
        let sql = format!(
            "select \
            id,\
            user_id,\
            state,\
            hold_pubkey,\
            brand,\
            holder_confirm_saved,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from device_info where {}",
            filter
        );
        let execute_res = PgLocalCli::query(sql.as_str()).await?;
        debug!("get device: raw sql {}", sql);
        let gen_view = |row: &Row| -> Result<DeviceInfoEntity> {
            Ok(DeviceInfoEntity {
                device_info: DeviceInfo {
                    id: row.get(0),
                    user_id: row.get::<usize, i64>(1) as u32,
                    state: row.get::<usize, String>(2).parse()?,
                    hold_pubkey: row.get(3),
                    brand: row.get(4),
                    holder_confirm_saved: row.get::<usize, bool>(5),
                },
                updated_at: row.get(6),
                created_at: row.get(7),
            })
        };

        execute_res.iter().map(gen_view).collect()
    }
    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update device_info set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = PgLocalCli::execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self) -> Result<()> {
        let DeviceInfo {
            id,
            user_id,
            state,
            hold_pubkey,
            brand: device_type,
            holder_confirm_saved,
        } = self.into_inner();
        let hold_pubkey: PsqlType = hold_pubkey.to_owned().into();

        let sql = format!(
            "insert into device_info (\
                id,\
                user_id,\
                state,\
                hold_pubkey,\
                brand,\
                holder_confirm_saved
        ) values ('{}',{},'{}',{},'{}',{});",
            id,
            user_id,
            state,
            hold_pubkey.to_psql_str(),
            device_type,
            holder_confirm_saved
        );
        debug!("row sql {} rows", sql);
        let _execute_res = PgLocalCli::execute(sql.as_str()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::general::{self, run_api_call, table_clear};

    use super::*;
    use common::log::init_logger;
    use std::env;

    #[tokio::test]
    async fn test_db_device_info() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();

        table_clear("device_info").await.unwrap();
        let task = async {
            let device = DeviceInfoEntity::new_with_specified("123", "Huawei", 1);
            device.insert().await.unwrap();
            let mut device_by_find =
                DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser("123", &1))
                    .await
                    .unwrap();
            println!("{:?}", device_by_find);
            //assert_eq!(device.device_info, device_by_find.device_info);

            device_by_find.device_info.user_id = 2;
            DeviceInfoEntity::update(
                DeviceInfoUpdater::State(SecretKeyState::Abandoned),
                DeviceInfoFilter::ByDeviceUser("123", &1),
            )
            .await
            .unwrap();
        };
        let (conn_ptr,_res) = run_api_call("", task).await.unwrap();
        general::commit().await.unwrap();
        general::clean_conn(conn_ptr);
    }
}
