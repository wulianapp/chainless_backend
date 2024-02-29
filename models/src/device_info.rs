extern crate rustc_serialize;

use std::fmt;
use std::fmt::Display;
use common::data_structures::device_info::DeviceInfo;
use postgres::Row;
//#[derive(Serialize)]
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use common::data_structures::SecretKeyState;
use serde::{Deserialize, Serialize};
use common::data_structures::wallet::{CoinTxStatus, StrategyMessageType};
use slog_term::PlainSyncRecordDecorator;
use common::data_structures::*;

use crate::{PsqlOp, vec_str2array_text};

use common::error_code::BackendError;


#[derive(Debug)]
pub enum DeviceInfoUpdater {
    State(SecretKeyState),
}

impl fmt::Display for DeviceInfoUpdater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoUpdater::State(new_state) =>  {
                format!("state='{}'", new_state.to_string())
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum DeviceInfoFilter {
    ByDeviceUser((String,u32)),
}

impl fmt::Display for DeviceInfoFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            DeviceInfoFilter::ByDeviceUser((device_id,user_id)) =>  
            format!("id='{}' and user_id={} ", device_id,user_id),
        };
        write!(f, "{}", description)
    }
}



#[derive(Deserialize, Serialize, Debug)]
pub struct DeviceInfoView{
    pub device_info: DeviceInfo,
    pub updated_at: String,
    pub created_at: String,
}

impl DeviceInfoView{
    pub fn new_with_specified(id:&str,
                              user_id:u32,
                              hold_pubkey: &str,
                              brand:&str,

    ) -> Self{
        DeviceInfoView{
            device_info: DeviceInfo{
                id: id.to_owned(),
                user_id,
                state: common::data_structures::DeviceState::Active,
                hold_pubkey:hold_pubkey.to_owned(),
                brand:brand.to_owned()
            },
            updated_at: "".to_string(),
            created_at: "".to_string()
        }
    }
}

impl PsqlOp for DeviceInfoView{

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
         cast(updated_at as text), \
         cast(created_at as text) \
         from device_info where {}",
            filter.to_string()
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row|{
            DeviceInfoView {
                device_info: DeviceInfo{
                    id: row.get(0),
                    user_id: row.get::<usize, i32>(1) as u32,
                    state:row.get::<usize, String>(2).parse().unwrap(),
                    hold_pubkey: row.get(3),
                    brand: row.get(4),
                },
                updated_at: row.get(5),
                created_at: row.get(6),
            }
        };

        Ok(execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<Self>>())
    }
    fn update(new_value: Self::UpdateContent, filter: Self::FilterContent) -> Result<(), BackendError> {
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
        } = &self.device_info;

        let sql = format!(
            "insert into device_info (\
                id,\
                user_id,\
                state,\
                hold_pubkey,\
                brand\
         ) values ('{}',{},'{}','{}','{}');",
         id,user_id,state.to_string(),hold_pubkey,device_type,
        );
        debug!("row sql {} rows", sql);
        let execute_res = crate::execute(sql.as_str())?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {

    use std::env;
    use super::*;
    use common::log::init_logger;

    #[test]
    fn test_db_device_info() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();

        crate::general::table_all_clear();

        let device = DeviceInfoView::new_with_specified(
            "123", 1, "01234567890abcd", "Huawei");
        device.insert().unwrap();
        let mut device_by_find = DeviceInfoView::find_single(
            DeviceInfoFilter::ByDeviceUser(("123".to_string(),1))).unwrap();
        println!("{:?}",device_by_find);
        assert_eq!(device.device_info,device_by_find.device_info);   

        device_by_find.device_info.user_id = 2;
        DeviceInfoView::update(
            DeviceInfoUpdater::State(SecretKeyState::Deprecated), 
            DeviceInfoFilter::ByDeviceUser(("123".to_string(),1))
        ).unwrap();
    }
}
