use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    btc_crypto::{self},
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::{AccountManagerError, BackendError},
    utils::math::coin_amount::display2raw,
};
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    device_info::{DeviceInfoEntity, DeviceInfoFilter},
    general::get_pg_pool_connect,
    wallet_manage_record::WalletManageRecordEntity,
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    utils::{token_auth, wallet_grades::query_wallet_grade},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};
use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, Clone, EnumString, Display)]
pub enum BindWay {
    Directly,
    Indirectly,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BindBtcAddressRequest {
    btc_address: String,
    sig: String,
    way: BindWay,
}

pub async fn req(req: HttpRequest, request_data: BindBtcAddressRequest) -> BackendRes<u8> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;

    let (_user, current_strategy, device) =
        get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = get_role(&current_strategy, device.hold_pubkey.as_deref());
    check_role(current_role, KeyRole2::Master)?;
    let main_account = get_main_account(user_id, &mut db_cli).await?;

    let BindBtcAddressRequest {
        btc_address,
        sig: _,
        way: bind_way,
    } = request_data;

    /***
    if !btc_crypto::verify(&user_id.to_string(), &sig, &btc_address)? {
        Err(BackendError::SigVerifyFailed)?;
    }
    ***/

    if !AirdropEntity::find(AirdropFilter::ByBtcAddress(&btc_address), &mut db_cli)
        .await?
        .is_empty()
    {
        Err(AirdropError::BtcAddressAlreadyUsed)?;
    }

    //todo: get kyc info
    let user_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByAccountId(&main_account), &mut db_cli).await?;
    if user_airdrop.airdrop.btc_address.is_some() {
        Err(AirdropError::AlreadyBindedBtcAddress)?;
    }

    let grade = match bind_way {
        BindWay::Directly => {
            let grade = query_wallet_grade(&btc_address).await?;
            AirdropEntity::update_single(
                AirdropUpdater::BtcAddressAndLevel(&btc_address, grade),
                AirdropFilter::ByAccountId(&main_account),
                &mut db_cli,
            )
            .await?;
            Some(grade)
        },
        BindWay::Indirectly => {
            AirdropEntity::update_single(
                AirdropUpdater::BtcAddress(&btc_address),
                AirdropFilter::ByAccountId(&main_account),
                &mut db_cli,
            )
            .await?;
            None
        }
    };

    Ok(grade)
}
