use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{AddressConvert, CoinTransaction, CoinTxStatus, CoinType};

use common::error_code::{BackendRes};
use crate::utils::token_auth;

use models::coin_transfer::CoinTxView;
use models::PsqlOp;

use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(req: HttpRequest, request_data: PreSendMoneyRequest) -> BackendRes<String> {
    //todo: allow master only
    let _user_id = token_auth::validate_credentials(&req)?;
    let PreSendMoneyRequest {
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
    let coin_info = CoinTxView::new_with_specified(coin_type, from, to, amount, coin_tx_raw, memo, expire_at);
    coin_info.insert()?;
    Ok(None::<String>)
}
