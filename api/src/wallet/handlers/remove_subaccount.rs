use std::collections::HashMap;

use actix_web::{web, HttpRequest};
use blockchain::coin::Coin;
use common::data_structures::wallet::get_support_coin_list;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
//use log::info;
use crate::utils::token_auth;
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::secret_store::{SecretFilter, SecretStoreView, SecretUpdater};
use models::{account_manager, secret_store, PsqlOp};
use tracing::info;
use crate::wallet::{RemoveSubaccountRequest, CreateMainAccountRequest, ReconfirmSendMoneyRequest};
use common::error_code::BackendError::ChainError;

pub async fn req(req: HttpRequest, request_data: RemoveSubaccountRequest) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let main_account = super::get_main_account(user_id)?;    
    let RemoveSubaccountRequest {
        account_id,
    } = request_data;
    super::have_no_uncompleted_tx(&main_account)?;
 
    let (_,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Master)?;

    if current_strategy.sub_confs.get(&account_id).is_none(){
        Err(WalletError::SubAccountNotExist(account_id.clone()))?;
    }

    //reserve one subaccount at least
    if current_strategy.sub_confs.len() == 1 {
        Err(WalletError::MustHaveSubaccount)?;
    }

    //check balance if is zero
    let coin_list = get_support_coin_list();
    for coin in &coin_list {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(coin.clone())?;
        if let Some(balance) = coin_cli.get_balance(&account_id).await?{
            if balance != "0".to_string(){
                Err(WalletError::BalanceMustBeZero)?;
            }
        }
    }


    models::general::transaction_begin()?;
    SecretStoreView::update(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&account_id),
    )?;
    let multi_cli = ContractClient::<MultiSig>::new()?;
    multi_cli.remove_subaccount(&main_account, &account_id).await?;
    models::general::transaction_commit()?;
    Ok(None::<String>)
}
