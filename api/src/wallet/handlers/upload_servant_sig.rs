use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::data_structures::wallet::{CoinTxStatus, CoinType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

use crate::wallet::uploadTxSignatureRequest;

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
    request_data: web::Json<uploadTxSignatureRequest>,
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
    let uploadTxSignatureRequest {
        tx_index,
        signature,
    } = request_data.0;

    //todo: validate signature

    let tx = models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    let mut signatures = tx.transaction.signatures.clone();
    signatures.push(signature);
    models::coin_transfer::CoinTxView::update(
        CoinTxUpdater::Signature(signatures),
        CoinTxFilter::ByTxIndex(tx_index),
    )?;
    //todo: collect enough signatures
    //let wallet_info = get_wallet(WalletFilter::ByUserId(user_id))?;
    //let wallet_info = &wallet_info.first().unwrap().wallet;

    //todo: checkout sig if is enough
    //first error deal with in models
    let multi_cli = blockchain::ContractClient::<MultiSig>::new();
    let strategy = multi_cli.get_strategy(&tx.transaction.from).await?.unwrap();
    let need = get_servant_need(
        &strategy.multi_sig_ranks,
        tx.transaction.coin_type,
        tx.transaction.amount
    ).await.unwrap();
    if tx.transaction.signatures.len() as u8 >= need {
        //区分receiver是否是子账户
        if let Some(_) = strategy.sub_confs.get(&tx.transaction.to){
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompletedAndReceiverIsSub),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }else{                
            models::coin_transfer::CoinTxView::update(
                CoinTxUpdater::Status(CoinTxStatus::SenderSigCompleted),
                CoinTxFilter::ByTxIndex(tx_index),
            )?;
        }

    }
    Ok(None::<String>)
}
