use actix_web::HttpRequest;

use airdrop::BtcGradeStatus;
use blockchain::airdrop::Airdrop;
use common::data_structures::KeyRole;
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{get_user_context, token_auth, wallet_grades::query_wallet_grade},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::BackendRes;
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
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;
    check_role(role, KeyRole::Master)?;
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


    //tmp: 等提前查等级的流程加上，再打开
    //防止一个金牌地址对多个账户小号打款评级
    match AirdropEntity::find_single(AirdropFilter::ByBtcAddress(&btc_address)).await {
        Ok(data) => {
            if data.into_inner().btc_grade_status == BtcGradeStatus::Reconfirmed{
                Err(AirdropError::BtcAddressAlreadyUsed)?;
            }
        },
        Err(e) if e.to_string().contains("DataNotFound") => {
            //do nothing
        },
        Err(e) => Err(e)?,
    }
    

    //todo: get kyc info
    let cli = ContractClient::<Airdrop>::new_update_cli().await.unwrap();
    let user_info = cli.get_user(&main_account).await?;
    if user_info.is_some() {
        Err(AirdropError::AlreadyClaimedDw20)?;
    }

    let grade = match bind_way {
        BindWay::Directly => {
            let grade = query_wallet_grade(&btc_address).await?;
            AirdropEntity::update_single(
                AirdropUpdater::BtcAddressAndLevel(&btc_address, Some(grade)),
                AirdropFilter::ByAccountId(&main_account),
            )
            .await?;
            Some(grade)
        }
        BindWay::Indirectly => {
            AirdropEntity::update_single(
                AirdropUpdater::BtcAddressAndLevel(&btc_address, None),
                AirdropFilter::ByAccountId(&main_account),
            )
            .await?;
            None
        }
    };

    Ok(grade)
}
