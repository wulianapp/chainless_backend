use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, PubkeySignInfo};
use common::data_structures::coin_transaction::{CoinSendStage};
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
        let _main_account = user.main_account;
        let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
        super::check_role(current_role,KeyRole2::Master)?;

    let ReactPreSendMoney {
        order_id,
        is_agreed,
    } = request_data;

    let coin_tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByOrderId(&order_id))?;
    if coin_tx.transaction.stage != CoinSendStage::SenderSigCompleted {
        Err(WalletError::TxStatusIllegal(coin_tx.transaction.stage,CoinSendStage::SenderSigCompleted))?;
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
            .map(|data| PubkeySignInfo {
                pubkey: data[..64].to_string(),

                signature: data[64..].to_string(),
            })
            .collect();

        //todo: replace with new api(gen_chain_tx) whereby avert tx expire
        let (tx_id, chain_raw_tx) = cli
            .gen_send_money_raw(
                servant_sigs,
                &coin_tx.transaction.from,
                &coin_tx.transaction.to,
                coin_tx.transaction.coin_type,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            ).await?;
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::ChainTxInfo(&tx_id, &chain_raw_tx, CoinSendStage::ReceiverApproved),
            CoinTxFilter::ByOrderId(&order_id),
        )?;
    } else {
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Stage(CoinSendStage::ReceiverRejected),
            CoinTxFilter::ByOrderId(&order_id),
        )?;
    };

    Ok(None)
}
