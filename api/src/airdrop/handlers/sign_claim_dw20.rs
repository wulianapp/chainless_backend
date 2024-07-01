use actix_web::HttpRequest;
use airdrop::{Airdrop, BtcGradeStatus};
use blockchain::{admin_sign, airdrop::Airdrop as ChainAirdrop};
use common::{data_structures::KeyRole, utils::time::now_millis};
use models::{
    airdrop::{AirdropEntity, AirdropFilter, AirdropUpdater},
    PsqlOp,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::utils::{get_user_context, token_auth};
use crate::wallet::handlers::*;
use blockchain::ContractClient;
use common::error_code::BackendRes;

#[derive(Deserialize, Serialize, Clone)]
struct ClaimDw20Param {
    account_id: String,
    btc_address: Option<String>,
    btc_level: u8,
    ref_account_id: String,
    deadline: u128,
}

pub fn gen_sign_msg(
    account_id: String,
    btc_address:  Option<String>,
    btc_level: u8,
    ref_account_id: String,
    deadline: u128,
) -> String {
    let param = ClaimDw20Param {
        account_id,
        btc_address,
        btc_level,
        ref_account_id,
        deadline,
    };
    serde_json::to_string(&param).unwrap()
}

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    let user_airdrop = AirdropEntity::find_single(AirdropFilter::ByUserId(&user_id))
        .await?
        .into_inner();
    if let Some(ref address) = user_airdrop.btc_address {
        if user_airdrop.btc_grade_status != BtcGradeStatus::Calculated {
            Err(AirdropError::BtcGradeStatusIllegal)?;
        }
        //一个btc地址允许被多账户评级，但是只允许一个最终上传
        let airdrops_by_btc = AirdropEntity::find(AirdropFilter::ByBtcAddress(address)).await?;
        for data in airdrops_by_btc {
            if data.airdrop.btc_grade_status == BtcGradeStatus::Reconfirmed {
                Err(AirdropError::BtcAddressAlreadyUsed)?;
            }
        }
    }

    AirdropEntity::update_single(
        AirdropUpdater::GradeStatus(BtcGradeStatus::Reconfirmed),
        AirdropFilter::ByAccountId(&main_account),
    )
    .await?;

    //上级必须也领过空投
    let cli = ContractClient::<ChainAirdrop>::new_query_cli().await?;
    let predecessor_airdrop_on_chain = cli.get_user(&user_airdrop.predecessor_account_id).await?;
    if predecessor_airdrop_on_chain.is_none() {
        Err(AirdropError::PredecessorHaveNotClaimAirdrop)?;
    }

    let Airdrop {predecessor_account_id,btc_address,btc_level,..} = user_airdrop;
    let deadline = now_millis() as u128 / 1000 + 60;
    let msg = gen_sign_msg(
        main_account,
         btc_address, 
         btc_level.unwrap_or_default(),
         predecessor_account_id,
         deadline
    );
    let sig = admin_sign(msg.as_bytes());
    debug!("successful claim dw20 txid {}", "ref_user");
    Ok(Some(sig))
}
