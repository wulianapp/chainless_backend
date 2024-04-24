extern crate rustc_serialize;

use common::data_structures::device_info::DeviceInfo;
use common::data_structures::wallet_namage_record::{WalletManageRecord, WalletOperateType};
use common::utils::math::generate_random_hex_string;
use postgres::Row;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
//#[derive(Serialize)]
use common::data_structures::SecretKeyState;
use common::data_structures::*;
use common::data_structures::{secret_store::SecretStore, SecretKeyType};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;
use derive_more::{AsRef, Deref};
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::TxStatusOnChain;


use crate::{vec_str2array_text, PsqlOp, PsqlType};
use anyhow::Result;
  

#[derive(Debug)]
pub enum WalletManageRecordUpdater<'a> {
    TxIds(&'a Vec<String>),
    Status(TxStatusOnChain),
}

impl fmt::Display for WalletManageRecordUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            WalletManageRecordUpdater::TxIds(ids) => {
            format!("tx_ids={} ", vec_str2array_text(ids.deref().to_owned()))
            }
            WalletManageRecordUpdater::Status(key) => {
                format!("status='{}' ",key.to_string())
            }
        };
        write!(f, "{}", description)
    }
}

//wallet_manage_record
#[derive(Clone, Debug)]
pub enum WalletManageRecordFilter<'b> {
    ByRecordId(&'b str),
    ByStatus(&'b TxStatusOnChain),
}

impl fmt::Display for WalletManageRecordFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::ByRecordId(record_id) => format!("record_id='{}' ", record_id),
            &Self::ByStatus(status) => {
                format!("status='{}' ", status.to_string())
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Deserialize, Serialize, Debug,AsRef,Clone)]
pub struct WalletManageRecordView {
    #[as_ref]
    pub record: WalletManageRecord,
    pub updated_at: String,
    pub created_at: String,
}

impl WalletManageRecordView {
    pub fn new_with_specified(
        user_id: &str, 
        operation_type: WalletOperateType, 
        operator_pubkey: &str,
        operator_device_id: &str,
        operator_device_brand: &str,
        tx_ids:Vec<String>
    
    ) -> Self {
        WalletManageRecordView {
            record: WalletManageRecord {
                record_id: generate_random_hex_string(64),
                user_id: user_id.to_string(),
                operation_type,
                operator_pubkey: operator_pubkey.to_string(),
                operator_device_id: operator_device_id.to_string(),
                operator_device_brand: operator_device_brand.to_string(),
                tx_ids,
                status: TxStatusOnChain::Pending,
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

//wallet_manage_history
impl PsqlOp for WalletManageRecordView {
    type UpdateContent<'a> = WalletManageRecordUpdater<'a>;
    type FilterContent<'b> = WalletManageRecordFilter<'b>;

    fn find(filter: Self::FilterContent<'_>) -> Result<Vec<Self>> {
        let sql = format!(
            "select 
            record_id,\
            user_id,\
            operation_type,\
            operator_pubkey,\
            operator_device_id,\
            operator_device_brand,\
            tx_ids,\
            status,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from wallet_manage_record where {}",
            filter
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get device: raw sql {}", sql);
        let gen_view = |row: &Row| -> Result<WalletManageRecordView> 
        {
            Ok(WalletManageRecordView {
                record: WalletManageRecord {
                    record_id: row.get(0),
                    user_id: row.get(1),
                    operation_type: row.get::<usize, String>(2).parse()?,
                    operator_pubkey: row.get(3),
                    operator_device_id: row.get(4),
                    operator_device_brand: row.get(5),
                    tx_ids: row.get::<usize, Vec<String>>(6),
                    status: row.get::<usize, String>(7).parse()?,
                },
                updated_at: row.get(8),
                created_at: row.get(9),
            })
        };

        execute_res
            .iter()
            .map(gen_view)
            .collect()
    }
    fn update(
        new_value: Self::UpdateContent<'_>,
        filter: Self::FilterContent<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update wallet_manage_record set {} where {}",
            new_value,
            filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
       //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    fn insert(&self) -> Result<()> {
        let WalletManageRecord {
            record_id,
            user_id,
            operation_type,
            operator_pubkey,
            operator_device_id,
            operator_device_brand,
            tx_ids,
            status,
        } = self.record.clone();
        let sql = format!(
            "insert into wallet_manage_record (\
                record_id,\
                user_id,\
                operation_type,\
                operator_pubkey,\
                operator_device_id,\
                operator_device_brand,\
                tx_ids,\
                status\
         ) values ('{}','{}','{}','{}','{}','{}',{},'{}');",
         record_id,
         user_id,
         operation_type.to_string(),
         operator_pubkey,
         operator_device_id,
         operator_device_brand,
         vec_str2array_text(tx_ids),
         status.to_string()
        );
        debug!("row sql {} rows", sql);
        let _execute_res = crate::execute(sql.as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use common::{data_structures::wallet_namage_record::WalletOperateType, log::init_logger};
    use std::env;

    #[test]
    fn test_db_wallet_manage_record() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let record = WalletManageRecordView::new_with_specified(
            "123",
            WalletOperateType::AddServant,
              "11111",
              "apple_device_id",
                "apple",
                vec![
                    "JBjvhpc3Uze77eVVoa4LvyAo1k6YQchMzsTd2pdADHvw".to_string(),
                "6sXahQSvCNkj7Y3uRhVcGHD1BPZcAHVtircBUj7L9NhY".to_string()]
        );
        record.insert().unwrap();

        let record_by_find =
        WalletManageRecordView::find_single(
            WalletManageRecordFilter::ByRecordId(&record.as_ref().record_id)).unwrap();
        println!("{:?}", record_by_find);

       // assert_eq!(record..record_id device_by_find.record);

       WalletManageRecordView::update(
            WalletManageRecordUpdater::Status(TxStatusOnChain::Successful),
            WalletManageRecordFilter::ByRecordId(&record.as_ref().record_id),
        )
        .unwrap();
    }
}
