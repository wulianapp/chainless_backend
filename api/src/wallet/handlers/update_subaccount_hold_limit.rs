use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::{
    data_structures::{wallet_namage_record::WalletOperateType, KeyRole2},
    error_code::BackendError,
    utils::math::coin_amount::display2raw,
};
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    wallet_manage_record::WalletManageRecordView,
    PsqlOp,
};

use crate::utils::token_auth;
use crate::wallet::UpdateSubaccountHoldLimitRequest;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(
    req: HttpRequest,
    request_data: UpdateSubaccountHoldLimitRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let UpdateSubaccountHoldLimitRequest { subaccount, limit } = request_data;
    let limit = display2raw(&limit).map_err(|err| BackendError::RequestParamInvalid(err))?;

    //add wallet info
    models::general::transaction_begin()?;
    let cli = ContractClient::<MultiSig>::new()?;

    let txid = cli
        .update_subaccount_hold_limit(&main_account, &subaccount, limit)
        .await?;
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::UpdateSubaccountHoldLimit,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert()?;
    models::general::transaction_commit()?;
    Ok(None::<String>)
}
