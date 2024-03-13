extern crate rustc_serialize;

use common::data_structures::device_info::DeviceInfo;
use postgres::Row;
use std::fmt;
use std::fmt::Display;
//#[derive(Serialize)]
use common::data_structures::wallet::CoinTxStatus;
use common::data_structures::SecretKeyState;
use common::data_structures::*;
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;

use crate::{vec_str2array_text, PsqlOp, PsqlType};

use common::error_code::BackendError;

#[derive(Debug)]
pub enum DeviceInfoUpdater {
    State(SecretKeyState),
    HolderSaved(bool),
    BecomeMaster(String),
    BecomeServant(String),
    BecomeUndefined(String),
}

impl fmt::Display for DeviceInfoUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoUpdater::State(new_state) => {
                format!("state='{}'", new_state.to_string())
            }
            DeviceInfoUpdater::HolderSaved(saved) => {
                format!("holder_confirm_saved={} ", saved)
            },
            DeviceInfoUpdater::BecomeMaster(key) => {
                format!("(hold_pubkey,holder_confirm_saved,key_role)=('{}',true,'Master') ", key)
            },
            DeviceInfoUpdater::BecomeServant(key) => {
                format!("(hold_pubkey,holder_confirm_saved,key_role)=('{}','true','Servant') ", key)
            },
            DeviceInfoUpdater::BecomeUndefined(key) => {
                format!("(hold_pubkey,holder_confirm_saved,key_role)=('{}',true,'Undefined') ", key)
            },
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum DeviceInfoFilter {
    ByUser(u32),
    ByDeviceUser(String, u32),
    ByUserDeviceHoldSecret(u32, String, bool),
    ByHoldKey(String),
}

impl fmt::Display for DeviceInfoFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoFilter::ByUser(user_id) => format!("user_id={} ", user_id),
            DeviceInfoFilter::ByDeviceUser(device_id, user_id) => {
                format!("id='{}' and user_id={} ", device_id, user_id)
            },
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

#[derive(Deserialize, Serialize, Debug)]
pub struct DeviceInfoView {
    pub device_info: DeviceInfo,
    pub updated_at: String,
    pub created_at: String,
}

impl DeviceInfoView {
    pub fn new_with_specified(
        id: &str,
        brand: &str,
        user_id: u32,
    ) -> Self {
        DeviceInfoView {
            device_info: DeviceInfo {
                id: id.to_owned(),
                user_id,
                state: common::data_structures::DeviceState::Active,
                hold_pubkey: None,
                brand: brand.to_owned(),
                holder_confirm_saved: false,
                key_role: KeyRole2::Undefined
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

impl PsqlOp for DeviceInfoView {
    type UpdateContent = DeviceInfoUpdater;
    type FilterContent = DeviceInfoFilter;

    fn find(filter: Self::FilterContent) -> Result<Vec<Self>, BackendError> {
        let sql = format!(
            "select 
            id,\
            user_id,\
            state,\
            hold_pubkey,\
            brand,\
            holder_confirm_saved,\
            key_role,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from device_info where {}",
            filter.to_string()
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row| DeviceInfoView {
            device_info: DeviceInfo {
                id: row.get(0),
                user_id: row.get::<usize, i32>(1) as u32,
                state: row.get::<usize, String>(2).parse().unwrap(),
                hold_pubkey: row.get(3),
                brand: row.get(4),
                holder_confirm_saved: row.get::<usize, bool>(5),
                key_role: row.get::<usize, String>(6).parse().unwrap(),
            },
            updated_at: row.get(7),
            created_at: row.get(8),
        };

        Ok(execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<Self>>())
    }
    fn update(
        new_value: Self::UpdateContent,
        filter: Self::FilterContent,
    ) -> Result<(), BackendError> {
        let sql = format!(
            "update device_info set {} where {}",
            new_value.to_string(),
            filter.to_string()
        );
        debug!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        debug!("success update orders {} rows", execute_res);
        Ok(())
    }

    fn insert(&self) -> Result<(), BackendError> {
        let DeviceInfo {
            id,
            user_id,
            state,
            hold_pubkey,
            brand: device_type,
            holder_confirm_saved,
            key_role,
        } = &self.device_info;
        let hold_pubkey: PsqlType = hold_pubkey.to_owned().into();

        let sql = format!(
            "insert into device_info (\
                id,\
                user_id,\
                state,\
                hold_pubkey,\
                brand,\
                holder_confirm_saved,\
                key_role\
         ) values ('{}',{},'{}',{},'{}',{},'{}');",
            id,
            user_id,
            state.to_string(),
            hold_pubkey.to_psql_str(),
            device_type,
            holder_confirm_saved,
            key_role.to_string()
        );
        debug!("row sql {} rows", sql);
        let _execute_res = crate::execute(sql.as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use common::log::init_logger;
    use std::env;

    #[test]
    fn test_db_device_info() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();

        crate::general::table_all_clear();

        let device =
            DeviceInfoView::new_with_specified("123", "Huawei", 1);
        device.insert().unwrap();
        let mut device_by_find =
            DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser("123".to_string(), 1))
                .unwrap();
        println!("{:?}", device_by_find);
        assert_eq!(device.device_info, device_by_find.device_info);

        device_by_find.device_info.user_id = 2;
        DeviceInfoView::update(
            DeviceInfoUpdater::State(SecretKeyState::Abandoned),
            DeviceInfoFilter::ByDeviceUser("123".to_string(), 1),
        )
        .unwrap();
    }
}
