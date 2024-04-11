use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, SignInfo};
use common::data_structures::wallet::{CoinTxStatus, TxType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use crate::wallet::ReconfirmSendMoneyRequest;
use common::error_code::{BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let _device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    let (user,current_strategy,device) = 
        super::get_session_state(user_id,&device_id).await?;
        let _main_account = user.main_account;
        let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
        super::check_role(current_role,KeyRole2::Master)?;

    //todo: check must be main device
    let ReconfirmSendMoneyRequest {
        tx_index,
        confirmed_sig,
    } = request_data.0;

    let coin_tx =
        models::coin_transfer::CoinTxView::find_single(CoinTxFilter::ByTxIndex(tx_index))?;
    //区分receiver是否是子账户
    let multi_cli = blockchain::ContractClient::<MultiSig>::new()?;
    let strategy = multi_cli.get_strategy(&coin_tx.transaction.from).await?.unwrap();
    if strategy.sub_confs.get(&coin_tx.transaction.to).is_some(){
        let servant_sigs = coin_tx
        .transaction
        .signatures
        .iter()
        .map(|data| data.parse().unwrap())
        .collect();
        //todo: unwrap()
        let master_sign : SignInfo= confirmed_sig.parse().unwrap();

        let tx_id =  multi_cli
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
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::ChainTxInfo(&tx_id, "", CoinTxStatus::SenderReconfirmed),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;

    }
    /***
     //跨链转出，在无链端按照普通转账处理
    else if coin_tx.transaction.tx_type == TxType::MainToBridge {
        let servant_sigs = coin_tx
        .transaction
        .signatures
        .iter()
        .map(|data| data.parse().unwrap())
        .collect();
        //todo: unwrap()
        let master_sign : SignInfo= confirmed_sig.parse().unwrap();

        let tx_id =  multi_cli
        .internal_withdraw(
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
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::ChainTxInfo(&tx_id, "", CoinTxStatus::SenderReconfirmed),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;

    }*/
    else{           
        blockchain::general::broadcast_tx_commit_from_raw2(
            coin_tx.transaction.chain_tx_raw.as_ref().unwrap(),
            &confirmed_sig,
        ).await;
        models::coin_transfer::CoinTxView::update(
            CoinTxUpdater::Status(CoinTxStatus::SenderReconfirmed),
            CoinTxFilter::ByTxIndex(tx_index),
        )?;     
    }
        
    Ok(None)
}
