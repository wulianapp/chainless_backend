
use actix_web::{HttpRequest};
use common::data_structures::wallet_namage_record::WalletOperateType;
use common::data_structures::{KeyRole};
use common::error_code::{BackendError, BackendRes, WalletError};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};

use models::wallet_manage_record::WalletManageRecordEntity;
//use log::info;

use crate::utils::{get_user_context, token_auth};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;





use models::{PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

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
    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;
    let CommitServantSwitchMasterRequest {
        add_key_raw,
        delete_key_raw,
        add_key_sig,
        delete_key_sig,
    } = request_data;

    let context = get_user_context(&user_id, &device_id).await?;
    let (main_account, _) = context.account_strategy()?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Servant)?;
    super::check_have_base_fee(&main_account).await?;
    super::have_no_uncompleted_tx(&main_account).await?;

    let multi_sig_cli = ContractClient::<MultiSig>::new_update_cli().await?;

    //外部注入和token解析结果对比
    let servant_pubkey =
        DeviceInfoEntity::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, &user_id))
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
        let _ =
            blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw, &add_key_sig).await;
        //更新设备信息
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeMaster(&servant_pubkey),
            DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
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
        let _ =
            blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw, &delete_key_sig)
                .await;
        DeviceInfoEntity::update_single(
            DeviceInfoUpdater::BecomeServant(&old_master),
            DeviceInfoFilter::ByHoldKey(&old_master),
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
            user_id,
            WalletOperateType::NewcomerSwitchMaster,
            &context.device.hold_pubkey.ok_or("")?,
            &context.device.id,
            &context.device.brand,
            vec![txid],
        );
        record.insert().await?;
    }
    Ok(None::<String>)
}
