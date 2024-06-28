use crate::utils::{get_main_account, judge_role_by_strategy, token_auth};
use actix_web::HttpRequest;
use anyhow::{anyhow, Result};

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::KeyRole;
use common::data_structures::{
    coin_transaction::{CoinSendStage, TxType},
    MT, TxStatusOnChain,
};

use common::error_code::BackendRes;
use common::error_code::{BackendError, WalletError};
use common::utils::math::coin_amount::raw2display;
use common::utils::math::hex_to_bs58;
use common::utils::time::now_millis;

use models::coin_transfer::{CoinTxEntity, CoinTxFilter};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};

use models::PsqlOp;
use serde::{Deserialize, Serialize};

use super::ServentSigDetail;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTxRequest {
    order_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeesDetailResponse {
    fee_coin: MT,
    fee_amount: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct GetTxResponse {
    pub order_id: String,
    pub tx_id: Option<String>,
    pub coin_type: MT,
    pub sender: String,
    pub receiver: String,
    pub receiver_account_id: String,
    pub amount: String,
    pub expire_at: u64,
    pub memo: Option<String>,
    pub stage: CoinSendStage,
    pub mt_tx_raw: String,
    pub signed_device: Vec<ServentSigDetail>,
    pub unsigned_device: Vec<ServentSigDetail>,
    pub tx_type: TxType,
    pub updated_at: String,
    pub created_at: String,
}

pub async fn req(req: HttpRequest, request_data: GetTxRequest) -> BackendRes<GetTxResponse> {
    let (user_id, _, _, _) = token_auth::validate_credentials(&req).await?;
    let main_account = get_main_account(&user_id).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let current_strategy = multi_sig_cli.get_strategy(&main_account).await?;

    let GetTxRequest { order_id } = request_data;
    let tx = CoinTxEntity::find_single(CoinTxFilter::ByOrderId(&order_id))
        .await
        .map_err(|e| {
            if e.to_string().contains("DBError::DataNotFound") {
                WalletError::OrderNotFound(order_id).into()
            } else {
                BackendError::InternalError(e.to_string())
            }
        })?;

    //获取已签名设备
    let mut signed_device = vec![];
    for sig in tx.transaction.signatures {
        let pubkey = sig[..64].to_string();
        let device = DeviceInfoEntity::find_single(DeviceInfoFilter::ByHoldKey(&pubkey))
            .await?
            .into_inner();
        let sig = ServentSigDetail {
            pubkey,
            device_id: device.id,
            device_brand: device.brand,
        };
        signed_device.push(sig);
    }

    //获取所有设备信息
    let all_device = DeviceInfoEntity::find(DeviceInfoFilter::ByUser(&user_id))
        .await?
        .into_iter()
        .filter(|x| {
            let role = judge_role_by_strategy(
                current_strategy.as_ref(),
                x.device_info.hold_pubkey.as_deref(),
            )
            .unwrap();
            role == KeyRole::Servant
        })
        .map(|d| ServentSigDetail {
            pubkey: d.device_info.hold_pubkey.unwrap(),
            device_id: d.device_info.id,
            device_brand: d.device_info.brand,
        })
        .collect::<Vec<ServentSigDetail>>();

    //获取未签名信息
    let unsigned_device = all_device
        .into_iter()
        .filter(|x| !signed_device.contains(x))
        .collect();

    let multi_sig_cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.to_owned()))?;

    let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &tx.transaction.coin_type,
        tx.transaction.amount,
    )
    .await;

    //如果接收方拒绝了，则该订单的最终状态位ReceiverRejected，而不是过期
    let stage = if tx.transaction.stage <= CoinSendStage::ReceiverApproved
        && now_millis() > tx.transaction.expire_at
    {
        CoinSendStage::MultiSigExpired
    } else {
        tx.transaction.stage
    };

    //对应需求，发起订单如果是通过联系方式，则展示联系方式
    let receiver = if let Some(contact) = tx.transaction.receiver_contact {
        contact
    } else {
        tx.transaction.receiver.clone()
    };

    let tx = GetTxResponse {
        order_id: tx.transaction.order_id,
        tx_id: tx.transaction.tx_id,
        coin_type: tx.transaction.coin_type,
        sender: tx.transaction.sender,
        receiver,
        receiver_account_id: tx.transaction.receiver,
        amount: raw2display(tx.transaction.amount),
        expire_at: tx.transaction.expire_at,
        memo: tx.transaction.memo,
        stage,
        mt_tx_raw: tx.transaction.coin_tx_raw,
        signed_device,
        unsigned_device,
        tx_type: tx.transaction.tx_type,
        updated_at: tx.updated_at,
        created_at: tx.created_at,
    };
    Ok(Some(tx))
}
