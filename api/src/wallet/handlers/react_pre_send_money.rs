use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, SignInfo};
use common::data_structures::wallet::{CoinTxStatus, CoinType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use crate::wallet::ReactPreSendMoney;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest, request_data: ReactPreSendMoney) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user,current_strategy,device) = 
        super::get_session_state(user_id,&device_id).await?;
        let main_account = user.main_account;
        super::have_no_uncompleted_tx(&main_account)?;
        let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
        super::check_role(current_role,KeyRole2::Master)?;

    let ReactPreSendMoney {
        tx_index,
        is_agreed,
    } = request_data;

    let coin_tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    if coin_tx.transaction.status != CoinTxStatus::SenderSigCompleted {
        Err(WalletError::TxStatusIllegal(coin_tx.transaction.status,CoinTxStatus::SenderSigCompleted))?;
    }

    //message max is 10，
    //let FinalizeSha = request_data.clone();
    if is_agreed {
        //todo:check user_id's main account_id is receiver

        let cli = blockchain::ContractClient::<MultiSig>::new()?;
        let _strategy = cli.get_strategy(&coin_tx.transaction.from).await.unwrap();
        let servant_sigs = coin_tx
            .transaction
            .signatures
            .iter()
            .map(|data| SignInfo {
                pubkey: data[..64].to_string(),

                signature: data[64..].to_string(),
            })
            .collect();

        //todo: replace with new api(gen_chain_tx) whereby avert tx expire
        let (tx_id, chain_raw_tx) = cli
            .gen_send_money_raw(
                tx_index as u64,
                servant_sigs,
                &coin_tx.transaction.from,
                &coin_tx.transaction.to,
                coin_tx.transaction.coin_type,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            )
            .await?;
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::ChainTxInfo(&tx_id, &chain_raw_tx, CoinTxStatus::ReceiverApproved),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    } else {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Status(CoinTxStatus::ReceiverRejected),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;
    };

    Ok(None::<String>)
}
