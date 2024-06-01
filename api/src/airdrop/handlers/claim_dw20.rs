use actix_web::{web, HttpRequest};

use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropEntity, AirdropFilter},
    general::get_pg_pool_connect,
    PsqlOp,
};
use tracing::{debug, info};

use crate::utils::{token_auth, wallet_grades::query_wallet_grade};
use crate::wallet::handlers::*;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device
    //todo: sync tx records after claim

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut db_cli).await?;

    let user_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id.to_string()), &mut db_cli)
            .await?;

    let cli = ContractClient::<ChainAirdrop>::new_update_cli().await?;
    let ref_user = cli
        .claim_dw20(
            &main_account,
            &user_airdrop.airdrop.predecessor_account_id,
            user_airdrop.airdrop.btc_address.as_deref(),
            user_airdrop.airdrop.btc_level.unwrap_or_default(),
        )
        .await?;
    debug!("successful claim dw20 txid {}", ref_user);
    Ok(None)
}
