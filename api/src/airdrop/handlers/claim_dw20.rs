use actix_web::{web, HttpRequest};

use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropFilter, AirdropView}, general::get_pg_pool_connect, PsqlOp
};
use tracing::{debug, info};

use crate::wallet::handlers::*;
use crate::wallet::UpdateStrategy;
use crate::{
    utils::{token_auth, wallet_grades::query_wallet_grade},
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device
    //todo: sync tx records after claim

    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut pg_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut pg_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut pg_cli).await?;

    let user_airdrop = AirdropView::find_single(
        AirdropFilter::ByUserId(&user_id.to_string()), 
        &mut pg_cli
    ).await?;
 

    let cli = ContractClient::<ChainAirdrop>::new().await?;
    let ref_user = cli
        .claim_dw20(
            &main_account,
            &user_airdrop.airdrop.predecessor_account_id,
            &user_airdrop.airdrop.btc_address.unwrap(),
            user_airdrop.airdrop.btc_level.unwrap()
        )
        .await?;
    debug!("successful claim dw20 txid {}", ref_user);
    Ok(None)
}
