use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, SignInfo};
use common::data_structures::wallet::{CoinTxStatus, CoinType};

use crate::wallet::ReactPreSendMoney;
use common::http::{token_auth, BackendRes};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdate};
use models::PsqlOp;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: ReactPreSendMoney,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let _user_id = token_auth::validate_credentials(&req)?;

    let ReactPreSendMoney {
        device_id,
        tx_index,
        is_agreed,
    } = request_data;
    //message max is 10，
    //let FinalizeSha = request_data.clone();
    if is_agreed {
        let coin_tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
        let cli = blockchain::ContractClient::<MultiSig>::new();
        let strategy = cli.get_strategy(&coin_tx.transaction.from).await.unwrap();
        let servant_sigs = coin_tx
            .transaction
            .signatures
            .iter()
            .map(|data| SignInfo {
                pubkey: data[..64].to_string(),

                signature: data[64..].to_string(),
            })
            .collect();

        let (tx_id, chain_raw_tx) = cli
            .gen_send_money_raw_tx(
                &coin_tx.transaction.from,
                &strategy.main_device_pubkey,
                servant_sigs,
                &coin_tx.transaction.from,
                &coin_tx.transaction.to,
                CoinType::DW20,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            )
            .await
            .unwrap();
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdate::ChainTxInfo(tx_id, chain_raw_tx, CoinTxStatus::ReceiverApproved),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    } else {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdate::Status(CoinTxStatus::ReceiverRejected),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    };
    Ok(None::<String>)
}
