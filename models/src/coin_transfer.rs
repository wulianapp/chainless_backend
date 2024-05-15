extern crate rustc_serialize;

use std::fmt;
use std::str::FromStr;

use common::utils::math::generate_random_hex_string;
use jsonrpc_http_server::jsonrpc_core::futures::future::OrElse;
use postgres::Row;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};

use crate::secret_store::{SecretFilter, SecretStoreView};
use crate::{vec_str2array_text, PsqlOp, PsqlType};
use anyhow::{Ok, Result};
use common::data_structures::coin_transaction::{CoinSendStage, CoinTransaction, TxRole, TxType};
use common::data_structures::{CoinType, TxStatusOnChain};

#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxView {
    pub transaction: CoinTransaction,
    pub updated_at: String,
    pub created_at: String,
}

impl CoinTxView {
    pub fn new_with_specified(
        coin_type: CoinType,
        from: String,
        to: String,
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
            from,
            to,
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
            reserved_field3: "".to_string(),
        };
        CoinTxView {
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
        /***
             *
             *    Normal,
        Forced,
        MainToSub,
        SubToMain,
        MainToBridge,
            */
        let description = match self {
            CoinTxFilter::ByChainStatus(status) => format!(
                "chain_status='{}'", status
            ),
            CoinTxFilter::ByUser(uid) => format!("sender='{}' or receiver='{}'", uid, uid),
            CoinTxFilter::BySender(uid) => format!("sender='{}'", uid),
            CoinTxFilter::ByReceiver(uid) => format!("receiver='{}' ", uid),
            CoinTxFilter::ByAccountPending(acc_id) => format!(
                "sender='{}' and stage in ('SenderSigCompleted','ReceiverApproved','ReceiverRejected','Created') or \
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

impl PsqlOp for CoinTxView {
    type UpdateContent<'a> = CoinTxUpdater<'a>;
    type FilterContent<'b> = CoinTxFilter<'b>;

    fn find(filter: Self::FilterContent<'_>) -> Result<Vec<CoinTxView>> {
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
         reserved_field3,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from coin_transaction where {}",
            filter
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_snapshot: raw sql {}", sql);

        let gen_view = |row: &Row| -> Result<CoinTxView> {
            Ok(CoinTxView {
                transaction: CoinTransaction {
                    order_id: row.get(0),
                    tx_id: row.get(1),
                    coin_type: CoinType::from_str(row.get::<usize, &str>(2))?,
                    from: row.get(3),
                    to: row.get(4),
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
                    reserved_field3: row.get(15),
                },
                updated_at: row.get(16),
                created_at: row.get(17),
            })
        };
        execute_res.iter().map(gen_view).collect()
    }

    fn update(update_data: CoinTxUpdater, filter: CoinTxFilter) -> Result<u64> {
        let sql = format!(
            "UPDATE coin_transaction SET {} ,updated_at=CURRENT_TIMESTAMP where {}",
            update_data, filter
        );
        info!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        //assert_ne!(execute_res, 0);
        info!("success update orders {} rows", execute_res);
        Ok(execute_res)
    }

    fn insert(&self) -> Result<()> {
        let CoinTransaction {
            order_id,
            tx_id,
            coin_type,
            from: sender,
            to: receiver,
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
            reserved_field3,
        } = self.transaction.clone();
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
         receiver_contact,\
         reserved_field3\
         ) values ('{}',{},'{}','{}','{}','{}','{}',{},'{}','{}',{},{},'{}','{}',{},'{}');",
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
            reserved_field3,
        );
        println!("row sql {} rows", sql);

        let execute_res = crate::execute(sql.as_str())?;
        info!("success insert {} rows", execute_res);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use common::data_structures::coin_transaction::CoinTransaction;
    use super::*;

    #[test]
    fn test_braced_models_coin_tx() {
        
        env::set_var("SERVICE_MODE", "test");
        crate::general::table_all_clear();

        let coin_tx = CoinTxView::new_with_specified(
            CoinType::BTC, 
            "1.test".to_string(), 
            "2.test".to_string(), 
            1, 
            "".to_string(),
             None, 
             1715740449000, 
             CoinSendStage::Created);

        println!("start insert");
        coin_tx.insert().unwrap();
        println!("start query");

        let _res = CoinTxView::find_single(CoinTxFilter::BySenderUncompleted("1.test")).unwrap();
        println!("start update");
        let _res = CoinTxView::update_single(
            CoinTxUpdater::Stage(CoinSendStage::MultiSigExpired),
            CoinTxFilter::ByOrderId(&coin_tx.transaction.order_id),
        ).unwrap();
        let res = CoinTxView::find_single(CoinTxFilter::ByOrderId(&coin_tx.transaction.order_id)).unwrap();
        println!("after update {:?}", res);

    }
}
