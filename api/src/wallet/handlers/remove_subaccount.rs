use std::collections::HashMap;

use actix_web::{web, HttpRequest};
use blockchain::coin::Coin;
use common::data_structures::get_support_coin_list;
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use models::general::get_db_pool_connect;
use models::wallet_manage_record::WalletManageRecordView;
//use log::info;
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, ReconfirmSendMoneyRequest, RemoveSubaccountRequest};
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use common::error_code::{BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::secret_store::{SecretFilter, SecretStoreView, SecretUpdater};
use models::{account_manager, secret_store, PsqlOp};
use tracing::info;

pub async fn req(req: HttpRequest, request_data: RemoveSubaccountRequest) -> BackendRes<String> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let RemoveSubaccountRequest { account_id } = request_data;
    super::have_no_uncompleted_tx(&main_account)?;

    let (_, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

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
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new_with_type(coin.clone()).await?;
        if let Some(balance) = coin_cli.get_balance(&account_id).await? {
            //当前不会出现小于1聪的情况，以后和第三方交互可能会有
            if balance != *"0" {
                Err(WalletError::BalanceMustBeZero)?;
            }
        }
    }
    
    println!("0001__");
    let mut conn = get_db_pool_connect()?;
    println!("0002__");
    let mut trans =  models::general::transaction_begin(&mut conn)?;
    println!("0003__");



    SecretStoreView::update_single_with_trans(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(sub_pubkey),
        &mut trans
    )?;
    let multi_cli = ContractClient::<MultiSig>::new().await?;
    let tx_id = multi_cli
        .remove_subaccount(&main_account, &account_id)
        .await?;

    //todo: generate txid before call contract
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::RemoveSubaccount,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert_with_trans(&mut trans)?;
    models::general::transaction_commit(trans)?;
    Ok(None::<String>)
}
