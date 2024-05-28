use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole2, SecretKeyState};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};
use models::general::{get_pg_pool_connect, transaction_begin};
use models::secret_store::{SecretFilter, SecretStoreEntity, SecretUpdater};
use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use common::error_code::BackendError::ChainError;
use models::account_manager::{get_next_uid, UserFilter, UserInfoEntity, UserUpdater};
use models::{account_manager, secret_store, PgLocalCli, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitServantSwitchMasterRequest {
    add_key_raw: String,
    delete_key_raw: String,
    add_key_sig: String,
    delete_key_sig: String,
}

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

    let mut db_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut db_cli = db_cli.begin().await?;

    //get user's main_account 、mater_key、current servant_key
    let main_account = super::get_main_account(user_id, &mut db_cli).await?;
    super::have_no_uncompleted_tx(&main_account, &mut db_cli).await?;
    let (_user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Servant)?;
    super::check_have_base_fee(&main_account, &mut db_cli).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new().await?;

    //todo: 检查防止用servantA的token操作servantB进行switch
    //外部注入和token解析结果对比
    let servant_pubkey = DeviceInfoEntity::find_single(
        DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
        &mut db_cli,
    )
    .await?
    .device_info
    .hold_pubkey
    .ok_or(BackendError::InternalError(
        "this haven't be servant yet".to_string(),
    ))?;
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;

    //get old_master
    let old_master = if master_list.len() == 1 {
        debug!("start switch servant to master");
        master_list[0].to_owned()
    } else if master_list.len() == 2 {
        warn!("unnormal account,it's account have 2 master");
        let mut local_list = master_list.clone();
        local_list.retain(|x| x.ne(&servant_pubkey));
        local_list[0].to_owned()
    } else {
        Err(BackendError::InternalError(
            "main account is unnormal".to_string(),
        ))?;
        unreachable!("");
    };

    //增加之前判断是否有
    if !master_list.contains(&servant_pubkey) {
        blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw, &add_key_sig).await;
        //更新设备信息
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeMaster(&servant_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, user_id),
            &mut db_cli,
        )
        .await?;
    } else {
        warn!("newcomer_pubkey<{}> already is master", servant_pubkey);
    }

    //除了同时包含servant_key和旧的master之外的情况全部认为异常不处理
    let master_list = multi_sig_cli.get_master_pubkey_list(&main_account).await?;
    if master_list.len() == 2
        && master_list.contains(&servant_pubkey)
        && master_list.contains(&old_master)
    {
        blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig).await;
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeServant(&old_master),
            DeviceInfoFilter::ByHoldKey(&old_master),
            &mut db_cli,
        )
        .await?;
    } else if master_list.len() == 1 && master_list.contains(&servant_pubkey) {
        warn!("old_master<{}>  is already deleted ", old_master);
    } else {
        //此类账户理论上不应该出现，除非是绕过了后台进行了操作
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
    if current_strategy.master_pubkey == servant_pubkey
        && current_strategy.servant_pubkeys.contains(&old_master)
    {
        warn!("servant adjustment  is already completed ");
    } else {
        current_strategy
            .servant_pubkeys
            .retain(|x| x != &servant_pubkey);

        current_strategy
            .servant_pubkeys
            .push(old_master.to_string());

        let txid = multi_sig_cli
            .update_servant_pubkey_and_master(
                &main_account,
                current_strategy.servant_pubkeys,
                servant_pubkey,
            )
            .await?;

        //前边两个用户管理的交互，可以无风险重试，暂时只有前两步完成，才能开始记录操作历史
        //从一开始就记录的话、状态管理太多
        let record = WalletManageRecordEntity::new_with_specified(
            &user_id.to_string(),
            WalletOperateType::NewcomerSwitchMaster,
            &device.hold_pubkey.ok_or("")?,
            &device.id,
            &device.brand,
            vec![txid],
        );
        record.insert(&mut db_cli).await?;
    }
    db_cli.commit().await?;
    Ok(None::<String>)
}
