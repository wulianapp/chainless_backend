extern crate rustc_serialize;

use async_trait::async_trait;
use common::data_structures::{
    bridge::{EthBridgeOrder, EthOrderStatus, OrderType as BridgeOrderType},
    CoinType,
};
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;
use std::fmt;
use std::fmt::Display;
use tokio_postgres::Row;

use crate::{vec_str2array_text, PgLocalCli, PsqlOp};
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EthBridgeOrderEntity {
    pub order: EthBridgeOrder,
    pub updated_at: String,
    pub created_at: String,
}

impl EthBridgeOrderEntity {
    pub fn into_inner(self) -> EthBridgeOrder {
        self.order
    }
}

#[derive(Debug)]
pub enum BridgeOrderUpdater<'a> {
    EncrypedPrikey(&'a str, &'a str),
    Status(EthOrderStatus),
}

impl fmt::Display for BridgeOrderUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            BridgeOrderUpdater::EncrypedPrikey(by_password, by_answer) => {
                format!(
                    "(encrypted_prikey_by_password,encrypted_prikey_by_answer)=('{}','{}')",
                    by_password, by_answer
                )
            }
            BridgeOrderUpdater::Status(status) => {
                format!("status='{}' ", status)
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum BridgeOrderFilter<'b> {
    ByTypeAndId(BridgeOrderType, &'b str),
    ByTypeAndAccountId(BridgeOrderType, &'b str),
    Limit(u32),
}

impl fmt::Display for BridgeOrderFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            BridgeOrderFilter::ByTypeAndId(order_type, id) => {
                format!("where order_type='{}' and id='{}' ", order_type, id)
            }
            BridgeOrderFilter::ByTypeAndAccountId(order_type, id) => format!(
                "where order_type='{}' and chainless_acc='{}' order by created_at desc",
                order_type, id
            ),
            BridgeOrderFilter::Limit(num) => format!("order by created_at desc limit {} ", num),
        };
        write!(f, "{}", description)
    }
}

impl EthBridgeOrderEntity {
    pub fn new_with_specified(
        id: &str,
        chainless_acc: &str,
        eth_addr: &str,
        order_type: BridgeOrderType,
        coin: CoinType,
        amount: u128,
        status: EthOrderStatus,
        height: u64,
    ) -> Self {
        EthBridgeOrderEntity {
            order: EthBridgeOrder {
                id: id.to_string(),
                order_type,
                chainless_acc: chainless_acc.to_owned(),
                eth_addr: eth_addr.to_owned(),
                coin,
                amount,
                status,
                height,
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

#[async_trait]
impl PsqlOp for EthBridgeOrderEntity {
    type UpdaterContent<'a> = BridgeOrderUpdater<'a>;
    type FilterContent<'b> = BridgeOrderFilter<'b>;
    async fn find(
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<Vec<EthBridgeOrderEntity>> {
        let sql = format!(
            "select 
            id,\
            order_type,\
            chainless_acc,\
            eth_addr,\
            coin,\
            amount,\
            status,\
            height,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from ethereum_bridge_order {}",
            filter
        );
        let execute_res = cli.query(sql.as_str()).await?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row| {
            Ok(EthBridgeOrderEntity {
                order: EthBridgeOrder {
                    id: row.get(0),
                    order_type: row.get::<usize, String>(1).parse()?,
                    chainless_acc: row.get(2),
                    eth_addr: row.get(3),
                    coin: row.get::<usize, String>(4).parse()?,
                    amount: row.get::<usize, String>(5).parse()?,
                    status: row.get::<usize, String>(6).parse()?,
                    height: row.get::<usize, i64>(7) as u64,
                },
                updated_at: row.get(8),
                created_at: row.get(9),
            })
        };

        execute_res.iter().map(gen_view).collect()
    }
    //没有更新的需求
    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        cli: &mut PgLocalCli<'_>,
    ) -> Result<u64> {
        let sql = format!(
            "update ethereum_bridge_order set {} ,updated_at=CURRENT_TIMESTAMP {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = cli.execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self, cli: &mut PgLocalCli<'_>) -> Result<()> {
        let EthBridgeOrder {
            id,
            order_type,
            chainless_acc,
            eth_addr,
            amount,
            coin,
            status,
            height,
        } = self.into_inner();
        let sql = format!(
            "insert into ethereum_bridge_order (\
                id,\
                order_type,\
                chainless_acc,\
                eth_addr,\
                coin,\
                amount,\
                status,\
                height
         ) values ('{}','{}','{}','{}','{}','{}','{}',{});",
            id, order_type, chainless_acc, eth_addr, coin, amount, status, height
        );
        debug!("row sql {} rows", sql);
        let _execute_res = cli.execute(sql.as_str()).await?;
        Ok(())
    }

    async fn delete(_filter: Self::FilterContent<'_>, _cli: &mut PgLocalCli<'_>) -> Result<()> {
        todo!()
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
    async fn test_db_bridge_order() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();

        let secret = EthBridgeOrderEntity::new_with_specified(
            "0123456789",
            "test.node0",
            "0x123",
            BridgeOrderType::Withdraw,
            CoinType::DW20,
            10000u128,
            EthOrderStatus::Pending,
            0u64,
        );
        secret.insert(&mut db_cli).await.unwrap();
        let find_res = EthBridgeOrderEntity::find_single(
            BridgeOrderFilter::ByTypeAndId(BridgeOrderType::Withdraw, "0123456789"),
            &mut db_cli,
        )
        .await
        .unwrap();
        println!("{:?}", find_res);
        assert_eq!(find_res.order.amount, 10000);
    }
}
