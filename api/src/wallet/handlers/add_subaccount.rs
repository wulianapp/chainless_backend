use std::collections::BTreeMap;

use actix_web::{HttpRequest};
use common::constants::{SUBACCOUNT_AMOUNT_LIMIT, SUBACCOUNT_TIME_LIMIT};
use common::data_structures::account_manager::UserInfo;
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::KeyRole;
use common::utils::math::coin_amount::display2raw;

use common::utils::time::{now_millis, DAY1};
use models::account_manager::{UserFilter, UserInfoEntity, UserUpdater};
use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;
use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::{MultiSig, SubAccConf};
use blockchain::ContractClient;




use common::error_code::{BackendError, BackendRes, WalletError};

use models::secret_store::SecretStoreEntity;
use models::{PsqlOp};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSubaccountRequest {
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_password: String,
    subaccount_prikey_encryped_by_answer: String,
    hold_value_limit: String,
}

//24小时内只能创建三次子账户
async fn update_add_record(user_info: &UserInfo) -> Result<(),BackendError>{
    let now = now_millis();
    let records_len = user_info.create_subacc_time.len() as u16;
    assert!(records_len <= SUBACCOUNT_AMOUNT_LIMIT);

    let mut times =  user_info.create_subacc_time.clone();
    if records_len == SUBACCOUNT_AMOUNT_LIMIT 
        && user_info.create_subacc_time[2] - now >= SUBACCOUNT_TIME_LIMIT{
       Err(WalletError::SubaccountCreateTooFrequently)?;
    }else if records_len == SUBACCOUNT_AMOUNT_LIMIT
        && user_info.create_subacc_time[2] - now < SUBACCOUNT_TIME_LIMIT{
        times.remove(0);
        times.push(now);  
    }else{
        times.push(now);
    }
    UserInfoEntity::update_single(
        UserUpdater::SubCreateRecords(times), 
    UserFilter::ById(&user_info.id)
    ).await?;

    Ok(())
}

pub async fn req(req: HttpRequest, request_data: AddSubaccountRequest) -> BackendRes<String> {
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let AddSubaccountRequest {
        subaccount_pubkey,
        subaccount_prikey_encryped_by_password,
        subaccount_prikey_encryped_by_answer,
        hold_value_limit,
    } = request_data;
    let hold_value_limit =
        display2raw(&hold_value_limit).map_err(|_e| WalletError::UnSupportedPrecision)?;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Master)?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let subaccount_id = super::gen_random_account_id(&multi_sig_cli).await?;

    let secret = SecretStoreEntity::new_with_specified(
        &subaccount_pubkey,
        user_id,
        &subaccount_prikey_encryped_by_password,
        &subaccount_prikey_encryped_by_answer,
    );
    secret.insert().await?;

    update_add_record(&context.user_info).await?;

    let multi_cli = ContractClient::<MultiSig>::new_update_cli().await?;
    let sub_confs = BTreeMap::from([(
        subaccount_id.as_str(),
        SubAccConf {
            pubkey: subaccount_pubkey,
            hold_value_limit,
        },
    )]);
    let txid = multi_cli.add_subaccount(&main_account, sub_confs).await?;

    let record = WalletManageRecordEntity::new_with_specified(
        user_id,
        WalletOperateType::AddSubaccount,
        &context.device.hold_pubkey.unwrap(),
        &context.device.id,
        &context.device.brand,
        vec![txid],
    );
    record.insert().await?;


    

    //info!("new wallet {:?}  successfully", user_info);
    Ok(None)
}
