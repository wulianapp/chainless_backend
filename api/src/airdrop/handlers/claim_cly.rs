use actix_web::HttpRequest;

use blockchain::airdrop::Airdrop;
use common::{data_structures::KeyRole, error_code::AccountManagerError};

use tracing::debug;

use crate::{
    utils::{get_user_context, token_auth},
    wallet::handlers::*,
};
use blockchain::ContractClient;
use common::error_code::BackendRes;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    check_role(role, KeyRole::Master)?;

    //领cly之前肯定已经领了dw20
    let cli = ContractClient::<Airdrop>::new_query_cli().await?;
    let user_airdrop_on_chain = cli
        .get_user(context.user_info.main_account.as_ref().unwrap())
        .await?;
    if user_airdrop_on_chain.is_none() {
        Err(AirdropError::HaveNotClaimAirdrop)?;
    }

    if !context.user_info.kyc_is_verified {
        Err(AccountManagerError::KYCNotRegister)?;
    }

    let mut cli = ContractClient::<Airdrop>::new_update_cli().await?;
    let receive_res = cli.claim_cly(&main_account).await?;
    debug!("successful claim air_reward {:?}", receive_res);
    Ok(None)
}
