use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{KeyRole2, PubkeySignInfo};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::get_pg_pool_connect;
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenSendMoneyRequest {
    order_id: String,
}

pub(crate) async fn req(req: HttpRequest, request_data: GenSendMoneyRequest) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let GenSendMoneyRequest { order_id } = request_data;

    let coin_tx = models::coin_transfer::CoinTxView::find_single(
        models::coin_transfer::CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
    .await?;

    let servant_sigs = coin_tx
        .transaction
        .signatures
        .iter()
        .map(|data| data.parse())
        .collect::<Result<Vec<_>, BackendError>>()?;

    //跨链的数据库存的是对应的eth地址，构造交易的时候需要改为桥地址
    let to = if coin_tx.transaction.tx_type == TxType::MainToBridge {
        common::env::CONF.bridge_near_contract.as_str()
    } else {
        coin_tx.transaction.to.as_str()
    };

    let cli = blockchain::ContractClient::<MultiSig>::new().await?;
    let (tx_id, chain_raw_tx) = cli
        .gen_send_money_raw(
            servant_sigs,
            &coin_tx.transaction.from,
            to,
            coin_tx.transaction.coin_type,
            coin_tx.transaction.amount,
            coin_tx.transaction.expire_at,
        )
        .await?;
    models::coin_transfer::CoinTxView::update_single(
        CoinTxUpdater::TxidTxRaw(&tx_id, &chain_raw_tx),
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
    .await?;
    Ok(Some(tx_id))
}
