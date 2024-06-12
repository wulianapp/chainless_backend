use std::collections::HashMap;

use actix_web::{web, HttpRequest};
use blockchain::coin::Coin;
use common::data_structures::get_support_coin_list;
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole, SecretKeyState};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;
use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};
use models::secret_store::{SecretFilter, SecretStoreEntity, SecretUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSubaccountRequest {
    account_id: String,
}

pub async fn req(req: HttpRequest, request_data: RemoveSubaccountRequest) -> BackendRes<String> {

    let (user_id, _, device_id, device_brand) =
        token_auth::validate_credentials(&req).await?;

    let RemoveSubaccountRequest { account_id } = request_data;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, current_strategy) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    //reserve one subaccount at least
    if current_strategy.sub_confs.len() == 1 {
        Err(WalletError::MustHaveSubaccount)?;
    }

    let sub_pubkey = match current_strategy.sub_confs.get(&account_id) {
        Some(conf) => &conf.pubkey,
        None => Err(WalletError::SubAccountNotExist(account_id.clone()))?,
    };

    //check balance if is zero
    let coin_list = get_support_coin_list();
    for coin in &coin_list {
        let coin_cli: ContractClient<Coin> =
            ContractClient::<Coin>::new_update_cli(coin.clone()).await?;
        if let Some(balance) = coin_cli.get_balance(&account_id).await? {
            //当前不会出现小于1聪的情况，以后和第三方交互可能会有
            if balance != *"0" {
                Err(WalletError::BalanceMustBeZero)?;
            }
        }
    }

    SecretStoreEntity::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(sub_pubkey),
       
    )
    .await?;
    let multi_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let tx_id = multi_cli
        .remove_subaccount(&main_account, &account_id)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::RemoveSubaccount,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert().await?;
    Ok(None::<String>)
}
