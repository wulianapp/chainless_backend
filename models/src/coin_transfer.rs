extern crate rustc_serialize;

use std::fmt;
use std::str::FromStr;

use postgres::Row;
//#[derive(Serialize)]
use serde::{Deserialize, Serialize};

use crate::secret_store::{SecretFilter, SecretStoreView};
use crate::{vec_str2array_text, PsqlOp, PsqlType};
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType};
use common::error_code::{BackendError, TxStatus};

#[derive(Deserialize, Serialize, Debug)]
pub struct CoinTxView {
    pub tx_index: u32,
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
        status: CoinTxStatus,
    ) -> Self {
        let coin_tx = CoinTransaction {
            tx_id: None,
            coin_type,
            from,
            to,
            amount,
            status,
            coin_tx_raw,
            chain_tx_raw: None,
            signatures: vec![],
            memo,
            expire_at,
        };
        CoinTxView {
            tx_index: 0,
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
    ByTxIndex(u32),
}

impl fmt::Display for CoinTxFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxFilter::ByUser(uid) => format!("sender='{}' or receiver='{}'", uid, uid),
            CoinTxFilter::BySender(uid) => format!("sender='{}'", uid),
            CoinTxFilter::ByReceiver(uid) => format!("receiver='{}' ", uid),
            CoinTxFilter::ByAccountPending(acc_id) => format!(
                "sender='{}' and status in ('SenderSigCompletedAndReceiverIsSub','ReceiverApproved','ReceiverRejected','Created') or \
                receiver='{}' and status in ('SenderSigCompleted')",
                acc_id, acc_id
            ),
            CoinTxFilter::BySenderUncompleted(acc_id) => format!(
                "sender='{}' and status in ('ReceiverApproved','ReceiverRejected','Created')",
                acc_id
            ),
            CoinTxFilter::ByTxIndex(tx_index) => format!("tx_index='{}' ", tx_index),
        };
        write!(f, "{}", description)
    }
}

#[derive(Clone, Debug)]
pub enum CoinTxUpdater<'a> {
    Status(CoinTxStatus),
    ChainTxInfo(&'a str, &'a str, CoinTxStatus),
    Signature(Vec<String>),
}

impl fmt::Display for CoinTxUpdater<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            CoinTxUpdater::Status(status) => format!("status='{}'", status.to_string()),
            CoinTxUpdater::ChainTxInfo(tx_id, chain_tx_raw, CoinTxStatus) => {
                format!(
                    "(tx_id,chain_tx_raw,status)=('{}','{}','{}')",
                    tx_id,
                    chain_tx_raw,
                    CoinTxStatus.to_string()
                )
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

    fn find(filter: Self::FilterContent<'_>) -> Result<Vec<CoinTxView>, BackendError> {
        let sql = format!(
            "select tx_index,\
         tx_id,\
         coin_type,\
         sender,\
         receiver,\
         amount,\
         expire_at,
         memo,
         status,\
         coin_tx_raw,\
         chain_tx_raw,\
         signatures,\
         cast(updated_at as text), \
         cast(created_at as text) \
         from coin_transaction where {}",
            filter.to_string()
        );
        let execute_res = crate::query(sql.as_str())?;
        debug!("get_snapshot: raw sql {}", sql);
        if execute_res.len() > 1 {
            //todo:throw error
            panic!("_tmp");
        }
        //let user_info_raw = execute_res.first().unwrap();

        let gen_view = |row: &Row| CoinTxView {
            tx_index: row.get::<usize, i32>(0) as u32,
            transaction: CoinTransaction {
                tx_id: row.get(1),
                coin_type: CoinType::from_str(row.get::<usize, &str>(2)).unwrap(),
                from: row.get(3),
                to: row.get(4),
                amount: u128::from_str(row.get::<usize, &str>(5)).unwrap(),
                expire_at: row.get::<usize, String>(6).parse().unwrap(),
                memo: row.get(7),
                status: CoinTxStatus::from_str(row.get::<usize, &str>(8)).unwrap(),
                coin_tx_raw: row.get(9),
                chain_tx_raw: row.get(10),
                signatures: row.get::<usize, Vec<String>>(11),
            },
            updated_at: row.get(12),
            created_at: row.get(13),
        };
        Ok(execute_res
            .iter()
            .map(|x| gen_view(x))
            .collect::<Vec<CoinTxView>>())
    }

    fn update(update_data: CoinTxUpdater, filter: CoinTxFilter) -> Result<(), BackendError> {
        let sql = format!(
            "UPDATE coin_transaction SET {} where {}",
            update_data.to_string(),
            filter.to_string()
        );
        info!("start update orders {} ", sql);
        let execute_res = crate::execute(sql.as_str())?;
        assert_ne!(execute_res, 0);
        info!("success update orders {} rows", execute_res);
        Ok(())
    }

    fn insert(&self) -> Result<(), BackendError> {
        let CoinTransaction {
            tx_id,
            coin_type,
            from: sender,
            to: receiver,
            amount,
            expire_at,
            memo,
            status,
            coin_tx_raw,
            chain_tx_raw,
            signatures,
        } = self.transaction.clone();
        let tx_id: PsqlType = tx_id.into();
        let chain_raw_data: PsqlType = chain_tx_raw.into();
        let memo: PsqlType = memo.into();

        //todo: amount specific type short or long
        let sql = format!(
            "insert into coin_transaction (tx_id,\
         coin_type,\
         sender,\
         receiver,\
         amount,\
         expire_at,\
         memo,\
         status,\
        coin_tx_raw,\
         chain_tx_raw,\
         signatures\
         ) values ({},'{}','{}','{}','{}','{}',{},'{}','{}',{},{});",
            tx_id.to_psql_str(),
            coin_type.to_string(),
            sender,
            receiver,
            amount.to_string(),
            expire_at.to_string(),
            memo.to_psql_str(),
            status.to_string(),
            coin_tx_raw,
            chain_raw_data.to_psql_str(),
            vec_str2array_text(signatures)
        );
        println!("row sql {} rows", sql);

        let execute_res = crate::execute(sql.as_str())?;
        info!("success insert {} rows", execute_res);

        Ok(())
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn test_braced_models_coin_tx() {
        /***
        env::set_var("SERVICE_MODE", "test");
        crate::general::table_all_clear();

        let coin_tx = CoinTransaction {
            tx_id: "123".to_string(),
            coin_type: CoinType::CLY,
            sender: 1,
            receiver: 2,
            amount: 0,
            status: CoinTxStatus::Created,
            raw_data: "123".to_string(),
            signatures: vec!["1".to_string()],
        };

        println!("start insert");
        single_insert(&coin_tx).unwrap();
        println!("start query");

        let _res = get_transactions(CoinTxFilter::ByUserPending(1));
        println!("start update");
        let test1 : CoinTxStatus= "123".parse()?;
        let _res = update_status(
            CoinTxStatus::ReceiverApproved,
            CoinTxFilter::ByUserPending(1),
        );
        let res = get_transactions(CoinTxFilter::ByUserPending(1));
        println!("after update {}", res.first().unwrap().transaction.status);

         */
    }
}
