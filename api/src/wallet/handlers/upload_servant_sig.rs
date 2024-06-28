use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{KeyRole, PubkeySignInfo};
use common::encrypt::ed25519_verify_hex;
use common::utils::time::now_millis;

use crate::utils::{get_user_context, token_auth};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::CoinTxEntity;
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadTxSignatureRequest {
    order_id: String,
    signature: String,
}

pub async fn req(req: HttpRequest, request_data: UploadTxSignatureRequest) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Servant)?;

    let UploadTxSignatureRequest {
        order_id,
        signature,
    } = request_data;

    //check signature's signer is  equal to device_holdkey
    let sign_info: PubkeySignInfo = signature.as_str().parse()?;
    if sign_info.pubkey != context.device.hold_pubkey.unwrap() {
        Err(BackendError::RequestParamInvalid(signature.clone()))?;
    }

    //todo: two update action is unnecessary
    let mut tx = CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id)).await?;

    let data = tx.transaction.coin_tx_raw;

    if !ed25519_verify_hex(&data, &sign_info.pubkey, &sign_info.signature)? {
        Err(BackendError::RequestParamInvalid(
            "siganature is illegal".to_string(),
        ))?;
    }

    if tx.transaction.stage != CoinSendStage::Created {
        Err(WalletError::TxStageIllegal(
            tx.transaction.stage,
            CoinSendStage::Created,
        ))?;
    }
    if now_millis() > tx.transaction.expire_at {
        Err(WalletError::TxExpired)?;
    }

    tx.transaction.signatures.push(signature);
    //fixme: repeat update twice
    CoinTxEntity::update_single(
        CoinTxUpdater::Signature(tx.transaction.signatures.clone()),
        CoinTxFilter::ByOrderId(&order_id),
    )
    .await?;

    //collect enough signatures
    let multi_cli = blockchain::ContractClient::<MultiSig>::new_update_cli().await?;

    let strategy = multi_cli
        .get_strategy(&tx.transaction.sender)
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
        if tx.transaction.tx_type == TxType::Forced {
            CoinTxEntity::update_single(
                CoinTxUpdater::Stage(CoinSendStage::ReceiverApproved),
                CoinTxFilter::ByOrderId(&order_id),
            )
            .await?;
        //非子账户非强制的话，签名收集够了则需要收款方进行确认
        } else {
            CoinTxEntity::update_single(
                CoinTxUpdater::Stage(CoinSendStage::SenderSigCompleted),
                CoinTxFilter::ByOrderId(&order_id),
            )
            .await?;
        }
    }
    Ok(None)
}
