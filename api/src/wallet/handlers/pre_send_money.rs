use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{AddressConvert, CoinTransaction, CoinTxStatus, CoinType};

use common::http::{token_auth, BackendRes};

use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(req: HttpRequest, request_data: PreSendMoneyRequest) -> BackendRes<String> {
    //todo: allow master only
    let _user_id = token_auth::validate_credentials(&req)?;
    let PreSendMoneyRequest {
        device_id: _,
        from,
        to,
        coin,
        amount,
        expire_at,
        memo,
    } = request_data;
    let coin_type = CoinType::from_account_str(&coin).unwrap();

    let cli = ContractClient::<MultiSig>::new();
    let coin_tx_raw = cli
        .gen_send_money_info(&from, &to, coin_type.clone(), amount, expire_at)
        .unwrap();
    let coin_tx = CoinTransaction {
        tx_id: None,
        coin_type,
        from: from,
        to: to,
        amount,
        status: CoinTxStatus::Created,
        coin_tx_raw,
        chain_tx_raw: None,
        signatures: vec![],
        memo,
        expire_at,
    };
    models::coin_transfer::single_insert(&coin_tx)?;
    Ok(None::<String>)
}
