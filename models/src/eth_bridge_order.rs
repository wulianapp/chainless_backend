extern crate rustc_serialize;

use common::data_structures::{bridge::{EthBridgeOrder, OrderType as BridgeOrderType}, CoinType};
use postgres::Row;
use serde::{Deserialize, Serialize};
use slog_term::PlainSyncRecordDecorator;
use std::fmt;
use std::fmt::Display;

use crate::{vec_str2array_text, PsqlOp};
use anyhow::{Result};

#[derive(Debug)]
pub enum BridgeOrderUpdater<'a> {
    EncrypedPrikey(&'a str, &'a str),
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
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum BridgeOrderFilter<'b> {
    ByTypeAndId(BridgeOrderType,&'b str),
    BySittingPubkey(&'b str),
}

impl fmt::Display for BridgeOrderFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            BridgeOrderFilter::ByTypeAndId(order_type,id) => 
            format!("order_type='{}' and id='{}' ",order_type.to_string(),id),
            BridgeOrderFilter::BySittingPubkey(key) => format!("state='Sitting' and pubkey='{}' ", key),
        };
        write!(f, "{}", description)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EthBridgeOrderView {
    pub order: EthBridgeOrder,
    pub updated_at: String,
    pub created_at: String,
}

impl EthBridgeOrderView {
    pub fn new_with_specified(
        id: &str,
        chainless_acc: &str,
        eth_add: &str,
        order_type: BridgeOrderType,
        coin: CoinType,
        amount: u128
    ) -> Self {
        EthBridgeOrderView {
            order: EthBridgeOrder {
                id: id.to_string(),
                order_type,
                chainless_acc: chainless_acc.to_owned(),
                eth_addr: eth_add.to_owned(),
                coin,
                amount,
                reserved_field1: "".to_string(),
                reserved_field2: "".to_string(),
                reserved_field3: "".to_string(),
            },
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

impl PsqlOp for EthBridgeOrderView {
    type UpdateContent<'a> = BridgeOrderUpdater<'a>;
    type FilterContent<'b> = BridgeOrderFilter<'b>;

    fn find(filter: BridgeOrderFilter) -> Result<Vec<EthBridgeOrderView>> {
        let sql = format!(
            "select 
            id,\
            order_type,\
            chainless_acc,\
            eth_addr,\
            coin,\
            amount,\
            reserved_field1,\
            reserved_field2,\
            reserved_field3,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from ethereum_bridge_order where {}",
            filter
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_secret: raw sql {}", sql);
        let gen_view = |row: &Row| {
            Ok(EthBridgeOrderView {
                order: EthBridgeOrder {
                    id: row.get(0),
                    order_type: row.get::<usize, String>(1).parse().unwrap(),
                    chainless_acc: row.get(2),
                    eth_addr: row.get(3),
                    coin: row.get::<usize, String>(4).parse().unwrap(),
                    amount: row.get::<usize, String>(5).parse().unwrap(),
                    reserved_field1: row.get(6),
                    reserved_field2: row.get(7),
                    reserved_field3: row.get(8),
                },
                updated_at: row.get(9),
                created_at: row.get(10),
            })
        };

        execute_res.iter().map(gen_view).collect()
    }
    //没有更新的需求
    fn update(new_value: BridgeOrderUpdater, filter: BridgeOrderFilter) -> Result<u64> {
        let sql = format!(
            "update ethereum_bridge_order set {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        debug!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        //assert_ne!(execute_res, 0);
        debug!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    fn insert(&self) -> Result<()> {
        let EthBridgeOrder {
            id,
            order_type,
            chainless_acc,
            eth_addr,
            amount,
            coin,
            reserved_field1,
            reserved_field2,
            reserved_field3,
        } = &self.order;
        let sql = format!(
            "insert into ethereum_bridge_order (\
                id,\
                order_type,\
                chainless_acc,\
                eth_addr,\
                coin,\
                amount,\
                reserved_field1,\
                reserved_field2,\
                reserved_field3\
         ) values ('{}','{}','{}','{}','{}','{}','{}','{}','{}');",
         id, 
         order_type.to_string(), 
         chainless_acc, 
         eth_addr, 
         coin,
         amount.to_string(),
         reserved_field1,reserved_field2,reserved_field3
        );
        debug!("row sql {} rows", sql);
        let _execute_res = crate::execute(sql.as_str())?;
        Ok(())
    }

    fn delete<T: Display>(_filter: T) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use common::log::init_logger;
    use std::env;

    #[test]
    fn test_db_bridge_order() {
        env::set_var("SERVICE_MODE", "test");
        init_logger();
        crate::general::table_all_clear();

        let secret =
            EthBridgeOrderView::new_with_specified("0123456789", "test.node0", 
            "0x123", BridgeOrderType::Withdraw,CoinType::DW20,10000u128);
        secret.insert().unwrap();
        let find_res =
            EthBridgeOrderView::find_single(
                BridgeOrderFilter::ByTypeAndId(BridgeOrderType::Withdraw,"0123456789")
            ).unwrap();
        println!("{:?}", find_res);
        assert_eq!(find_res.order.amount, 10000);
    }
}
