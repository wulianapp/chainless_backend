use actix_web::{web, HttpRequest};

use blockchain::{
    airdrop::Airdrop,
    multi_sig::{MultiSig, MultiSigRank},
};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole},
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

use crate::{
    utils::{get_user_context, token_auth},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest) -> BackendRes<String> {

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;
    if !context.user_info.kyc_is_verified {
        Err(AccountManagerError::KYCNotRegister)?;
    }

    //todo: check if claimed already
    let cli = ContractClient::<Airdrop>::new_update_cli().await?;
    let receive_res = cli.claim_cly(&main_account).await?;
    debug!("successful claim air_reward {:?}", receive_res);
    Ok(None)
}
