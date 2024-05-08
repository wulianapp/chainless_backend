use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{KeyRole2, PubkeySignInfo};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

use crate::wallet::UploadTxSignatureRequest;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<UploadTxSignatureRequest>,
) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (_user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Servant)?;

    let UploadTxSignatureRequest {
        order_id,
        signature,
    } = request_data.0;

    //check signature's signer is  equal to device_holdkey
    let sign_info: PubkeySignInfo = signature.as_str().parse()?;
    if sign_info.pubkey != device.hold_pubkey.unwrap() {
        Err(BackendError::RequestParamInvalid(signature.clone()))?;
    }
    //todo: two update action is unnecessary
    let mut tx =
        models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByOrderId(&order_id))?;
    if tx.transaction.stage != CoinSendStage::Created {
        Err(WalletError::TxStageIllegal(
            tx.transaction.stage,
            CoinSendStage::Created,
        ))?;
    }

    tx.transaction.signatures.push(signature);
    //fixme: repeat update twice
    models::general::transaction_begin()?;
    models::coin_transfer::CoinTxView::update_single(
        CoinTxUpdater::Signature(tx.transaction.signatures.clone()),
        CoinTxFilter::ByOrderId(&order_id),
    )?;

    //collect enough signatures
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;

    let strategy = multi_cli
        .get_strategy(&tx.transaction.from)
        .await?
        .ok_or("from not found")?;

    let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &tx.transaction.coin_type,
        tx.transaction.amount,
    )
    .await;
    if tx.transaction.signatures.len() as u8 >= need_sig_num {
        //区分receiver是否是子账户
        //给子账户转是relayer进行签名，不需要生成tx_raw
        if tx.transaction.tx_type == TxType::MainToSub
            || tx.transaction.tx_type == TxType::MainToBridge
        {
            models::coin_transfer::CoinTxView::update_single(
                CoinTxUpdater::Stage(CoinSendStage::ReceiverApproved),
                CoinTxFilter::ByOrderId(&order_id),
            )?;
        //给其他主账户转是用户自己签名，需要生成tx_raw
        } else if tx.transaction.tx_type == TxType::Forced {
            let cli = ContractClient::<MultiSig>::new()?;

            let servant_sigs = tx
                .transaction
                .signatures
                .iter()
                .map(|data| PubkeySignInfo {
                    pubkey: data[..64].to_string(),
                    signature: data[64..].to_string(),
                })
                .collect();
            let (tx_id, chain_tx_raw) = cli
                .gen_send_money_raw(
                    servant_sigs,
                    &tx.transaction.from,
                    &tx.transaction.to,
                    tx.transaction.coin_type,
                    tx.transaction.amount,
                    tx.transaction.expire_at,
                )
                .await?;
            models::coin_transfer::CoinTxView::update_single(
                CoinTxUpdater::ChainTxInfo(&tx_id, &chain_tx_raw, CoinSendStage::ReceiverApproved),
                CoinTxFilter::ByOrderId(&order_id),
            )?;
        //非子账户非强制的话，签名收集够了则需要收款方进行确认
        } else {
            models::coin_transfer::CoinTxView::update_single(
                CoinTxUpdater::Stage(CoinSendStage::SenderSigCompleted),
                CoinTxFilter::ByOrderId(&order_id),
            )?;
        }
    }
    models::general::transaction_commit()?;
    Ok(None)
}
