use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretStoreView, SecretUpdater};
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{
    CommitNewcomerSwitchMasterRequest, CommitServantSwitchMasterRequest,
    CreateMainAccountRequest, ReconfirmSendMoneyRequest,
};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use common::error_code::BackendError::ChainError;

//todo：这里后边加上channel的异步处理，再加一张表用来记录所有非交易的交互的状态，先pending，再更新状态
pub(crate) async fn req(
    req: HttpRequest,
    request_data: CommitServantSwitchMasterRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _device_brand) = token_auth::validate_credentials2(&req)?;
    let CommitServantSwitchMasterRequest {
        add_key_raw,
        delete_key_raw,
        add_key_sig,
        delete_key_sig,
    } = request_data;

    //get user's main_account 、mater_key、current servant_key
    let main_account = super::get_main_account(user_id)?;
    super::have_no_uncompleted_tx(&main_account)?;
    let (user,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Servant)?;

    let servant_pubkey =
        DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?
            .device_info
            .hold_pubkey
            .ok_or(BackendError::InternalError(
                "this haven't be servant yet".to_string(),
            ))?;
    let multi_sig_cli = ContractClient::<MultiSig>::new()?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;
    if master_list.len() != 1 {
        error!("unnormal account， it's account have more than 1 master");
        return Err(common::error_code::BackendError::InternalError(
            "".to_string(),
        ));
    }
    let old_master = master_list.first().unwrap();

    models::general::transaction_begin()?;

    //增加之前判断是否有
    if !master_list.contains(&servant_pubkey) {
        blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw, &add_key_sig).await;
        //更新设备信息
        DeviceInfoView::update(
            DeviceInfoUpdater::BecomeMaster(&servant_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
        )?;
    } else {
        warn!("newcomer_pubkey<{}> already is master", servant_pubkey);
    }

    //除了同时包含servant_key和旧的master之外的情况全部认为异常不处理
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;
    if master_list.len() == 2
        && master_list.contains(&servant_pubkey)
        && master_list.contains(old_master)
    {
        blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig).await;
        DeviceInfoView::update(
            DeviceInfoUpdater::BecomeServant(&old_master),
            DeviceInfoFilter::ByHoldKey(&old_master),
        )?;
        
    }else if master_list.len() == 1 && master_list.contains(&servant_pubkey){
        warn!("old_master<{}>  is already deleted ", old_master);
    } else {
        //此类账户理论上不应该出现，除非是绕过了后台进行了操作
        error!("main account is unnormal");
        Err(BackendError::InternalError(
            "main account is unnormal".to_string(),
        ))?;
    }

    
    //operate servant: delete older and than add new
    let mut current_strategy = multi_sig_cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::MainAccountNotExist(main_account.clone()))?;

    //maybe is unnecessary
    if &current_strategy.master_pubkey == old_master 
        && current_strategy.servant_pubkeys.contains(&servant_pubkey){
        warn!("servant adjustment  is already completed ");
    }else{
        current_strategy
            .servant_pubkeys
            .retain(|x| x != &servant_pubkey);

        current_strategy
            .servant_pubkeys
            .push(old_master.to_string());

        multi_sig_cli
            .update_servant_pubkey_and_master(&main_account, current_strategy.servant_pubkeys,servant_pubkey)
            .await?;
    }
    models::general::transaction_commit()?;
    Ok(None::<String>)
}
