use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{KeyRole2, PubkeySignInfo};
use common::encrypt::{ed25519_verify_hex, ed25519_verify_raw};
use common::utils::time::now_millis;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use tracing::warn;

use crate::utils::token_auth;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::{PgLocalCli, PsqlOp};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadTxSignatureRequest {
    order_id: String,
    signature: String,
}

pub async fn req(req: HttpRequest, request_data: UploadTxSignatureRequest) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    let (_user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Servant)?;

    let UploadTxSignatureRequest {
        order_id,
        signature,
    } = request_data;

    //check signature's signer is  equal to device_holdkey
    let sign_info: PubkeySignInfo = signature.as_str().parse()?;
    if sign_info.pubkey != device.hold_pubkey.unwrap() {
        Err(BackendError::RequestParamInvalid(signature.clone()))?;
    }

    //todo: two update action is unnecessary
    let mut tx = models::coin_transfer::CoinTxEntity::find_single(
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
    .await?;

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
    models::coin_transfer::CoinTxEntity::update_single(
        CoinTxUpdater::Signature(tx.transaction.signatures.clone()),
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
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
        if tx.transaction.tx_type == TxType::MainToSub
            || tx.transaction.tx_type == TxType::MainToBridge
        {
            models::coin_transfer::CoinTxEntity::update_single(
                CoinTxUpdater::Stage(CoinSendStage::ReceiverApproved),
                CoinTxFilter::ByOrderId(&order_id),
                &mut db_cli,
            )
            .await?;
        //给其他主账户转是用户自己签名，需要生成tx_raw
        } else if tx.transaction.tx_type == TxType::Forced {
            //todo: 83~102 line is redundant，txid生成在gen_send_money的时候进行了
            let cli = ContractClient::<MultiSig>::new_update_cli().await?;
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
                    &tx.transaction.sender,
                    &tx.transaction.receiver,
                    tx.transaction.coin_type,
                    tx.transaction.amount,
                    tx.transaction.expire_at,
                )
                .await?;

            models::coin_transfer::CoinTxEntity::update_single(
                CoinTxUpdater::ChainTxInfo(&tx_id, &chain_tx_raw, CoinSendStage::ReceiverApproved),
                CoinTxFilter::ByOrderId(&order_id),
                &mut db_cli,
            )
            .await?;
        //非子账户非强制的话，签名收集够了则需要收款方进行确认
        } else {
            models::coin_transfer::CoinTxEntity::update_single(
                CoinTxUpdater::Stage(CoinSendStage::SenderSigCompleted),
                CoinTxFilter::ByOrderId(&order_id),
                &mut db_cli,
            )
            .await?;
        }
    }
    db_cli.commit().await?;
    Ok(None)
}
