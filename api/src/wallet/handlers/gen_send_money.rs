use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::{KeyRole, PubkeySignInfo};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::secret_store::SecretStoreEntity;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenSendMoneyRequest {
    order_id: String,
}

pub(crate) async fn req(req: HttpRequest, request_data: GenSendMoneyRequest) -> BackendRes<String> {
    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req, &mut db_cli).await?;

    let context = get_user_context(&user_id, &device_id, &mut db_cli).await?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;

    let GenSendMoneyRequest { order_id } = request_data;

    let coin_tx = models::coin_transfer::CoinTxEntity::find_single(
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
        coin_tx.transaction.receiver.as_str()
    };

    let cli = blockchain::ContractClient::<MultiSig>::new_query_cli().await?;
    let (tx_id, chain_raw_tx) = cli
        .gen_send_money_raw(
            servant_sigs,
            &coin_tx.transaction.sender,
            to,
            coin_tx.transaction.coin_type,
            coin_tx.transaction.amount,
            coin_tx.transaction.expire_at,
        )
        .await?;
    models::coin_transfer::CoinTxEntity::update_single(
        CoinTxUpdater::TxidTxRaw(&tx_id, &chain_raw_tx),
        CoinTxFilter::ByOrderId(&order_id),
        &mut db_cli,
    )
    .await?;
    Ok(Some(tx_id))
}
