extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::device_info::DeviceInfo;
use common::data_structures::wallet_namage_record::{WalletManageRecord, WalletOperateType};
use common::utils::math::generate_random_hex_string;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use tokio_postgres::Row;
//#[derive(Serialize)]
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::secret_store::SecretStore;
use common::data_structures::SecretKeyState;
use common::data_structures::TxStatusOnChain;
use common::data_structures::*;
use derive_more::{AsRef, Deref};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;

use crate::{vec_str2array_text, PgLocalCli, PsqlOp, PsqlType};
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, AsRef, Clone)]
pub struct WalletManageRecordEntity {
    #[as_ref]
    pub record: WalletManageRecord,
    pub updated_at: String,
    pub created_at: String,
}

impl WalletManageRecordEntity {
    pub fn into_inner(self) -> WalletManageRecord {
        self.record
    }
}

#[derive(Debug)]
pub enum WalletManageRecordUpdater<'a> {
    TxIds(&'a Vec<String>),
    Status(TxStatusOnChain),
}

impl fmt::Display for WalletManageRecordUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            WalletManageRecordUpdater::TxIds(ids) => {
                format!("tx_ids={} ", vec_str2array_text(ids.to_vec()))
            }
            WalletManageRecordUpdater::Status(key) => {
                format!("status='{}' ", key)
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
                format!("status='{}' ", status)
            }
        };
        write!(f, "{}", description)
    }
}

impl WalletManageRecordEntity {
    pub fn new_with_specified(
        user_id: u32,
        operation_type: WalletOperateType,
        operator_pubkey: &str,
        operator_device_id: &str,
        operator_device_brand: &str,
        tx_ids: Vec<String>,
    ) -> Self {
        WalletManageRecordEntity {
            record: WalletManageRecord {
                record_id: generate_random_hex_string(64),
                user_id,
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
#[async_trait]
impl PsqlOp for WalletManageRecordEntity {
    type UpdaterContent<'a> = WalletManageRecordUpdater<'a>;
    type FilterContent<'b> = WalletManageRecordFilter<'b>;

    async fn find(filter: Self::FilterContent<'_>, cli: &mut PgLocalCli<'_>) -> Result<Vec<Self>> {
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
        let execute_res = cli.query(sql.as_str()).await?;
        debug!("get device: raw sql {}", sql);
        let gen_view = |row: &Row| -> Result<WalletManageRecordEntity> {
            Ok(WalletManageRecordEntity {
                record: WalletManageRecord {
                    record_id: row.get(0),
                    user_id: row.get::<usize, i64>(1) as u32,
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

        execute_res.iter().map(gen_view).collect()
    }
    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update wallet_manage_record set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = cli.execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self, cli: &mut PgLocalCli<'_>) -> Result<()> {
        let WalletManageRecord {
            record_id,
            user_id,
            operation_type,
            operator_pubkey,
            operator_device_id,
            operator_device_brand,
            tx_ids,
            status,
        } = self.into_inner();
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
            operation_type,
            operator_pubkey,
            operator_device_id,
            operator_device_brand,
            vec_str2array_text(tx_ids),
            status
        );
        debug!("row sql {} rows", sql);
        let _execute_res = cli.execute(sql.as_str()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::general::{get_pg_pool_connect, transaction_begin, transaction_commit};
    use common::log::init_logger;
    use std::env;
    use tokio_postgres::types::ToSql;

    #[tokio::test]
    async fn test_db_wallet_manage_record() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear().await;

        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();
        let mut db_cli = db_cli.begin().await.unwrap();

        let record = WalletManageRecordEntity::new_with_specified(
            123,
            WalletOperateType::AddServant,
            "11111",
            "apple_device_id",
            "apple",
            vec![
                "JBjvhpc3Uze77eVVoa4LvyAo1k6YQchMzsTd2pdADHvw".to_string(),
                "6sXahQSvCNkj7Y3uRhVcGHD1BPZcAHVtircBUj7L9NhY".to_string(),
            ],
        );
        let record_id = record.record.record_id.clone();
        record.insert(&mut db_cli).await.unwrap();

        let record_by_find = WalletManageRecordEntity::find_single(
            WalletManageRecordFilter::ByRecordId(&record_id),
            &mut db_cli,
        )
        .await
        .unwrap();
        println!("{:?}", record_by_find);

        // assert_eq!(record..record_id device_by_find.record);

        WalletManageRecordEntity::update(
            WalletManageRecordUpdater::Status(TxStatusOnChain::Successful),
            WalletManageRecordFilter::ByRecordId(&record_id),
            &mut db_cli,
        )
        .await
        .unwrap();
    }
}
