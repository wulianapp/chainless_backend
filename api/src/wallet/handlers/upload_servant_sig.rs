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

async fn get_servant_need(
    strategy: &Vec<MultiSigRank>,
    _coin: CoinType,
    amount: u128,
) -> Option<u8> {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let coin_price = 1;
    let transfer_value = amount * coin_price;
    strategy
        .iter()
        .find(|&rank| transfer_value >= rank.min && transfer_value < rank.max_eq)
        .map(|rank| rank.sig_num)
}


pub async fn req(
    req: HttpRequest,
    request_data: web::Json<UploadTxSignatureRequest>,
) -> BackendRes<String> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Servant {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Servant,
        ))?;
    }

    //todo: check must be main device
    let UploadTxSignatureRequest {
        tx_index,
        signature,
    } = request_data.0;

    //todo: two update action is unnecessary
    let mut tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    tx.transaction.signatures.push(signature);
    models::coin_transfer::CoinTxView::update(
        CoinTxUpdater::Signature(tx.transaction.signatures.clone()),
        CoinTxFilter::ByTxIndex(tx_index),
    )?;

    //collect enough signatures
    let multi_cli = blockchain::ContractClient::<MultiSig>::new();
    let strategy = multi_cli.get_strategy(&tx.transaction.from).await?.unwrap();
    let need = get_servant_need(
        &strategy.multi_sig_ranks,
        tx.transaction.coin_type.clone(),
        tx.transaction.amount
    ).await.unwrap();
    if tx.transaction.signatures.len() as u8 >= need {
        //区分receiver是否是子账户
        if tx.transaction.tx_type == TxType::ToSub{
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompletedAndReceiverIsSub),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }else if tx.transaction.tx_type == TxType::Forced{
            let cli = ContractClient::<MultiSig>::new();
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
            .await
            .unwrap()
            .unwrap();

            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::ChainTxInfo(&tx_id, &chain_tx_raw, CoinTxStatus::ReceiverApproved),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }else{                
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompleted),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }

    }
    Ok(None)
}
