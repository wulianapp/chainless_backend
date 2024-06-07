use actix_web::{web, HttpRequest};

use blockchain::multi_sig::MultiSig;
use common::data_structures::coin_transaction::CoinSendStage;
use common::data_structures::{KeyRole, PubkeySignInfo, TxStatusOnChain};
use common::encrypt::{ed25519_verify_hex, ed25519_verify_raw};
use common::utils::time::now_millis;
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use tracing::{debug, info};

use crate::utils::{get_user_context, token_auth};
use common::error_code::{to_internal_error, BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::{PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconfirmSendMoneyRequest {
    order_id: String,
    confirmed_sig: String,
}

pub async fn req(req: HttpRequest, request_data: ReconfirmSendMoneyRequest) -> BackendRes<String> {
    //todo:check user_id if valid

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;
    let ReconfirmSendMoneyRequest {
        order_id,
        confirmed_sig,
    } = request_data;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let (_main_account, current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;

    let coin_tx = models::coin_transfer::CoinTxEntity::find_single(
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
    .await?;
    if now_millis() > coin_tx.transaction.expire_at {
        Err(WalletError::TxExpired)?;
    }
    //区分receiver是否是子账户
    let multi_cli = blockchain::ContractClient::<MultiSig>::new_update_cli().await?;
    let strategy = multi_cli
        .get_strategy(&coin_tx.transaction.sender)
        .await?
        .ok_or(BackendError::InternalError(
            "main_account not found".to_string(),
        ))?;

    //todo: check sig before push it to blockchain
    if confirmed_sig.len() != 192 && confirmed_sig.len() != 128 {
        Err(BackendError::RequestParamInvalid(
            "confirmed_sig is invalid".to_string(),
        ))?;
    }

    if strategy
        .sub_confs
        .get(&coin_tx.transaction.receiver)
        .is_some()
    {
        info!("coin_tx {:?} is a tx which send money to sub", coin_tx);

        //提前进行签名校验
        let data = coin_tx.transaction.coin_tx_raw;
        let sign_info: PubkeySignInfo = confirmed_sig.as_str().parse()?;
        if !ed25519_verify_hex(&data, &sign_info.pubkey, &sign_info.signature)? {
            Err(BackendError::RequestParamInvalid(
                "siganature is illegal".to_string(),
            ))?;
        }

        let servant_sigs = coin_tx
            .transaction
            .signatures
            .iter()
            .map(|data| data.parse())
            .collect::<Result<Vec<PubkeySignInfo>, _>>()?;
        let master_sign: PubkeySignInfo = confirmed_sig.parse()?;

        let tx_id = multi_cli
            .internal_transfer_main_to_sub(
                master_sign,
                servant_sigs,
                &coin_tx.transaction.sender,
                &coin_tx.transaction.receiver,
                coin_tx.transaction.coin_type,
                coin_tx.transaction.amount,
                coin_tx.transaction.expire_at,
            )
            .await?;

        //todo:txid?
        models::coin_transfer::CoinTxEntity::update_single(
            CoinTxUpdater::TxidStageChainStatus(
                &tx_id,
                CoinSendStage::SenderReconfirmed,
                TxStatusOnChain::Pending,
            ),
            CoinTxFilter::ByOrderId(&order_id),
            &mut db_cli,
        )
        .await?;
    } else {
        //提前进行签名校验
        let data = coin_tx
            .transaction
            .tx_id
            .ok_or(BackendError::InternalError("".to_string()))?;
        let pubkey = current_strategy.master_pubkey;
        if !ed25519_verify_hex(&data, &pubkey, &confirmed_sig)? {
            Err(BackendError::RequestParamInvalid(
                "siganature is illegal".to_string(),
            ))?;
        }

        //跨链转出，在无链端按照普通转账处理
        blockchain::general::broadcast_tx_commit_from_raw2(
            coin_tx.transaction.chain_tx_raw.as_ref().ok_or("")?,
            &confirmed_sig,
        )
        .await?;
        models::coin_transfer::CoinTxEntity::update_single(
            CoinTxUpdater::StageChainStatus(
                CoinSendStage::SenderReconfirmed,
                TxStatusOnChain::Pending,
            ),
            CoinTxFilter::ByOrderId(&order_id),
            &mut db_cli,
        )
        .await?;
    }
    db_cli.commit().await?;
    Ok(None)
}
