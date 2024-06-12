extern crate rustc_serialize;

use std::fmt;
use std::str::FromStr;

use async_trait::async_trait;
use common::utils::math::generate_random_hex_string;
use jsonrpc_http_server::jsonrpc_core::futures::future::OrElse;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use crate::secret_store::{SecretFilter, SecretStoreEntity};
use crate::{vec_str2array_text, PgLocalCli, PgLocalCli2, PsqlOp, PsqlType};
use anyhow::{Ok, Result};
use common::data_structures::coin_transaction::{CoinSendStage, CoinTransaction, TxRole, TxType};
use common::data_structures::{CoinType, TxStatusOnChain};

#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxEntity {
    pub transaction: CoinTransaction,
    pub updated_at: String,
    pub created_at: String,
}

impl CoinTxEntity {
    pub fn into_inner(self) -> CoinTransaction {
        self.transaction
    }
}

impl CoinTxEntity {
    pub fn new_with_specified(
        coin_type: CoinType,
        sender: String,
        receiver: String,
        amount: u128,
        coin_tx_raw: String,
        memo: Option<String>,
        expire_at: u64,
        stage: CoinSendStage,
    ) -> Self {
        let coin_tx = CoinTransaction {
            order_id: generate_random_hex_string(32),
            tx_id: None,
            coin_type,
            sender,
            receiver,
            amount,
            stage,
            coin_tx_raw,
            chain_tx_raw: None,
            signatures: vec![],
            memo,
            expire_at,
            tx_type: TxType::Normal,
            chain_status: TxStatusOnChain::NotLaunch,
            receiver_contact: None,
        };
        CoinTxEntity {
            transaction: coin_tx,
            updated_at: "".to_string(),
            created_at: "".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CoinTxFilter<'b> {
    ByUser(u32),
    BySender(u32),
    ByReceiver(u32),
    ByAccountPending(&'b str),
    BySenderUncompleted(&'b str),
    //todo: replace with u128
    ByOrderId(&'b str),
    //role1,role1_account,role2_account,per_page,page
    ByTxRolePage(TxRole, &'b str, Option<&'b str>, u32, u32),
    ByChainStatus(TxStatusOnChain),
}

impl fmt::Display for CoinTxFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxFilter::ByChainStatus(status) => format!(
                "chain_status='{}'", status
            ),
            CoinTxFilter::ByUser(uid) => format!("sender='{}' or receiver='{}'", uid, uid),
            CoinTxFilter::BySender(uid) => format!("sender='{}'", uid),
            CoinTxFilter::ByReceiver(uid) => format!("receiver='{}' ", uid),
            CoinTxFilter::ByAccountPending(acc_id) => format!(
                "sender='{}' and stage in ('SenderSigCompleted','ReceiverApproved','Created') or \
                receiver='{}' and stage in ('SenderSigCompleted','ReceiverApproved')",
                acc_id, acc_id
            ),
            CoinTxFilter::BySenderUncompleted(acc_id) => format!(
                "sender='{}' and stage in ('ReceiverApproved','ReceiverRejected','Created','SenderSigCompleted')",
                acc_id
            ),
            CoinTxFilter::ByOrderId(id) => format!("order_id='{}' ", id),
            CoinTxFilter::ByTxRolePage(role,account,counterparty,per_page,page) => {
                let offset = if *page == 1u32 {
                    0
                }else{
                    (page - 1u32) * per_page - 1u32
                };
                //过滤自己和交易对手方
                match  counterparty {
                    Some(counterparty_account) => format!(
                        "{}='{}' and {}='{}' order by updated_at desc limit {} offset {}",
                        role,account,role.counterparty(),counterparty_account,per_page,offset
                    ),
                    None => format!(
                        "{}='{}' order by updated_at desc limit {} offset {}",
                        role,account,per_page,offset
                    ),
                }
            }
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum CoinTxUpdater<'a> {
    Stage(CoinSendStage),
    StageChainStatus(CoinSendStage, TxStatusOnChain),
    TxidStageChainStatus(&'a str, CoinSendStage, TxStatusOnChain),
    ChainTxInfo(&'a str, &'a str, CoinSendStage),
    TxidTxRaw(&'a str, &'a str),
    Signature(Vec<String>),
}

impl fmt::Display for CoinTxUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxUpdater::Stage(stage) => format!("stage='{}'", stage),
            CoinTxUpdater::StageChainStatus(stage, status) => {
                format!("(stage,chain_status)=('{}','{}')", stage, status)
            }
            CoinTxUpdater::TxidStageChainStatus(txid, stage, status) => format!(
                "(tx_id,stage,chain_status)=('{}','{}','{}')",
                txid, stage, status
            ),
            CoinTxUpdater::ChainTxInfo(tx_id, chain_tx_raw, stage) => {
                format!(
                    "(tx_id,chain_tx_raw,stage)=('{}','{}','{}')",
                    tx_id, chain_tx_raw, stage
                )
            }
            CoinTxUpdater::TxidTxRaw(tx_id, chain_tx_raw) => {
                format!("(tx_id,chain_tx_raw)=('{}','{}')", tx_id, chain_tx_raw,)
            }
            CoinTxUpdater::Signature(sigs) => {
                format!("signatures={}", vec_str2array_text(sigs.to_owned()))
            }
        };
        write!(f, "{}", description)
    }
}
#[async_trait]
impl PsqlOp for CoinTxEntity {
    type UpdaterContent<'a> = CoinTxUpdater<'a>;
    type FilterContent<'b> = CoinTxFilter<'b>;

    async fn find(
        filter: Self::FilterContent<'_>,
        
    ) -> Result<Vec<CoinTxEntity>> {
        let sql = format!(
            "select order_id,\
         tx_id,\
         coin_type,\
         sender,\
         receiver,\
         amount,\
         expire_at,
         memo,
         stage,\
         coin_tx_raw,\
         chain_tx_raw,\
         signatures,\
         tx_type,\
         chain_status,\
         receiver_contact,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from coin_transaction where {}",
            filter
        );
        let execute_res = PgLocalCli2::query(sql.as_str()).await?;
        debug!("get_snapshot: raw sql {}", sql);

        let gen_view = |row: &Row| -> Result<CoinTxEntity> {
            Ok(CoinTxEntity {
                transaction: CoinTransaction {
                    order_id: row.get(0),
                    tx_id: row.get(1),
                    coin_type: CoinType::from_str(row.get::<usize, &str>(2))?,
                    sender: row.get(3),
                    receiver: row.get(4),
                    amount: u128::from_str(row.get::<usize, &str>(5))?,
                    expire_at: row.get::<usize, String>(6).parse()?,
                    memo: row.get(7),
                    stage: row.get::<usize, &str>(8).parse()?,
                    coin_tx_raw: row.get(9),
                    chain_tx_raw: row.get(10),
                    signatures: row.get::<usize, Vec<String>>(11),
                    tx_type: row.get::<usize, &str>(12).parse()?,
                    chain_status: row.get::<usize, &str>(13).parse()?,
                    receiver_contact: row.get::<usize, Option<String>>(14),
                },
                updated_at: row.get(15),
                created_at: row.get(16),
            })
        };
        execute_res.iter().map(gen_view).collect()
    }

    async fn update(
        new_value: Self::UpdaterContent<'_>,
        filter: Self::FilterContent<'_>,
        
    ) -> Result<u64> {
        let sql = format!(
            "UPDATE coin_transaction SET {} ,updated_at=CURRENT_TIMESTAMP where {}",
            new_value, filter
        );
        info!("start update orders {} ", sql);
        let execute_res = PgLocalCli2::execute(sql.as_str()).await?;
        //assert_ne!(execute_res, 0);
        info!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    async fn insert(self) -> Result<()> {
        let CoinTransaction {
            order_id,
            tx_id,
            coin_type,
            sender,
            receiver,
            amount,
            expire_at,
            memo,
            stage,
            coin_tx_raw,
            chain_tx_raw,
            signatures,
            tx_type,
            chain_status,
            receiver_contact,
        } = self.into_inner();
        let tx_id: PsqlType = tx_id.into();
        let chain_raw_data: PsqlType = chain_tx_raw.into();
        let memo: PsqlType = memo.into();
        let receiver_contact: PsqlType = receiver_contact.into();

        //todo: amount specific type short or long
        let sql = format!(
            "insert into coin_transaction (order_id,
                tx_id,\
         coin_type,\
         sender,\
         receiver,\
         amount,\
         expire_at,\
         memo,\
         stage,\
        coin_tx_raw,\
         chain_tx_raw,\
         signatures,\
         tx_type,\
         chain_status,\
         receiver_contact
         ) values ('{}',{},'{}','{}','{}','{}','{}',{},'{}','{}',{},{},'{}','{}',{});",
            order_id,
            tx_id.to_psql_str(),
            coin_type,
            sender,
            receiver,
            amount,
            expire_at,
            memo.to_psql_str(),
            stage,
            coin_tx_raw,
            chain_raw_data.to_psql_str(),
            vec_str2array_text(signatures),
            tx_type,
            chain_status,
            receiver_contact.to_psql_str(),
        );
        println!("row sql {} rows", sql);

        let execute_res = PgLocalCli2::execute(sql.as_str()).await?;
        info!("success insert {} rows", execute_res);

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
    async fn test_db_coin_transfer() {
        env::set_var("SERVICE_MODE", "test");
        crate::general::table_all_clear().await;
        let mut db_cli: PgLocalCli = get_pg_pool_connect().await.unwrap();

        let coin_tx = CoinTxEntity::new_with_specified(
            CoinType::BTC,
            "1.test".to_string(),
            "2.test".to_string(),
            1,
            "".to_string(),
            None,
            1715740449000,
            CoinSendStage::Created,
        );

        let order_id = coin_tx.transaction.order_id.clone();
        println!("start insert");
        coin_tx.insert().await.unwrap();
        println!("start query");

        let _res =
            CoinTxEntity::find_single(CoinTxFilter::BySenderUncompleted("1.test"))
                .await
                .unwrap();
        println!("start update");
        CoinTxEntity::update_single(
            CoinTxUpdater::Stage(CoinSendStage::MultiSigExpired),
            CoinTxFilter::ByOrderId(&order_id),
           
        )
        .await
        .unwrap();
        let res = CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id))
            .await
            .unwrap();
        println!("after update {:?}", res);
    }
}
