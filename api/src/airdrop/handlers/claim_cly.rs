use actix_web::{web, HttpRequest};

use blockchain::{
    airdrop::Airdrop,
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use lettre::transport::smtp::client;
use models::{
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{utils::token_auth, wallet::handlers::*};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut db_cli).await?;

    //todo: check if claimed already
    let cli = ContractClient::<Airdrop>::new().await?;
    let receive_res = cli.claim_cly(&main_account).await?;
    debug!("successful claim air_reward {:?}", receive_res);
    Ok(None)
}
