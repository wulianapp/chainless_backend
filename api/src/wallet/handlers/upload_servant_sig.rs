use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank, SignInfo};
use blockchain::ContractClient;
use common::data_structures::wallet::{CoinTxStatus, CoinType, TxType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use common::error_code::{BackendRes, WalletError};
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
    let (_user,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Servant)?;
    
    let UploadTxSignatureRequest {
        tx_index,
        signature,
    } = request_data.0;

    //todo: two update action is unnecessary
    let mut tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    if tx.transaction.status != CoinTxStatus::Created {
        Err(WalletError::TxStatusIllegal(tx.transaction.status,CoinTxStatus::Created))?;
    }
    
    tx.transaction.signatures.push(signature);
    models::coin_transfer::CoinTxView::update(
        CoinTxUpdater::Signature(tx.transaction.signatures.clone()),
        CoinTxFilter::ByTxIndex(tx_index),
    )?;

    //collect enough signatures
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;

    let strategy = multi_cli.get_strategy(&tx.transaction.from)
    .await?
    .unwrap();

    let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &tx.transaction.coin_type,
        tx.transaction.amount
    ).await;
    if tx.transaction.signatures.len() as u8 >= need_sig_num {
        //区分receiver是否是子账户
        //给子账户转是relayer进行签名，不需要生成tx_raw
        if tx.transaction.tx_type == TxType::ToSub{
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompletedAndReceiverIsSub),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        //给其他主账户转是用户自己签名，需要生成tx_raw    
        }else if tx.transaction.tx_type == TxType::Forced{
            let cli = ContractClient::<MultiSig>::new()?;
    
            let servant_sigs = tx
            .transaction
            .signatures
            .iter()
            .map(|data| SignInfo {
                pubkey: data[..64].to_string(),
                signature: data[64..].to_string(),
            })
            .collect();
            let (tx_id, chain_tx_raw) = cli
            .gen_send_money_raw(
                tx.tx_index as u64,
                servant_sigs,
                &tx.transaction.from,
                &tx.transaction.to,
                tx.transaction.coin_type,
                tx.transaction.amount,
                tx.transaction.expire_at,
            )
            .await?;
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::ChainTxInfo(&tx_id, &chain_tx_raw, CoinTxStatus::ReceiverApproved),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        //非子账户非强制的话，签名收集够了则需要收款方进行确认        
        }else{                
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompleted),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }

    }
    Ok(None)
}
