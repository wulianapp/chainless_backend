use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig};
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::{KeyRole2, PubkeySignInfo, TxStatusOnChain};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::{debug, info};

use crate::utils::token_auth;
use crate::wallet::ReconfirmSendMoneyRequest;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: ReconfirmSendMoneyRequest,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let ReconfirmSendMoneyRequest {
        order_id,
        confirmed_sig,
    } = request_data;

    let coin_tx =
        models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByOrderId(&order_id))?;
    //区分receiver是否是子账户
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;
    let strategy = multi_cli
        .get_strategy(&coin_tx.transaction.from)
        .await?
        .ok_or(BackendError::InternalError("main_account not found".to_string()))?;
    
    //todo: check sig before push it to blockchain
    if confirmed_sig.len() != 192 && confirmed_sig.len() != 128{
        Err(BackendError::RequestParamInvalid("confirmed_sig is invalid".to_string()))?;
    }

    models::general::transaction_begin()?;
    if strategy.sub_confs.get(&coin_tx.transaction.to).is_some() {
        info!("coin_tx {:?} is a tx which send money to sub", coin_tx);
        let servant_sigs = coin_tx
            .transaction
            .signatures
            .iter()
            .map(|data| data.parse())
            .collect::<Result<Vec<PubkeySignInfo>,_>>()?;
        //todo: unwrap()
        let master_sign: PubkeySignInfo = confirmed_sig.parse()?;

        let tx_id = multi_cli
            .internal_transfer_main_to_sub(
                master_sign,
                servant_sigs,
                &coin_tx.transaction.from,
                &coin_tx.transaction.to,
                coin_tx.transaction.coin_type,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            )
            .await?;

        //todo:txid?
        models::coin_transfer::CoinTxView::update_single(
            CoinTxUpdater::TxidStageChainStatus(
                &tx_id,
                CoinSendStage::SenderReconfirmed,
                TxStatusOnChain::Pending,
            ),
            CoinTxFilter::ByOrderId(&order_id),
        )?;
    } else {
        //跨链转出，在无链端按照普通转账处理
        blockchain::general::broadcast_tx_commit_from_raw2(
            coin_tx.transaction.chain_tx_raw.as_ref().unwrap(),
            &confirmed_sig,
        )
        .await;
        models::coin_transfer::CoinTxView::update_single(
            CoinTxUpdater::StageChainStatus(
                CoinSendStage::SenderReconfirmed,
                TxStatusOnChain::Pending,
            ),
            CoinTxFilter::ByOrderId(&order_id),
        )?;
    }
    models::general::transaction_commit()?;
    Ok(None)
}
