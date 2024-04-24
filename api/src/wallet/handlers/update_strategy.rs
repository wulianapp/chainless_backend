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
use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest, request_data: web::Json<UpdateStrategy>) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;

    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    let UpdateStrategy { strategy } = request_data.0;

    //fixme:
    let strategy = strategy
        .into_iter()
        .map(|x| -> Result<MultiSigRank, String> {
            let rank = MultiSigRank {
                min: display2raw(&x.min)?,
                max_eq: display2raw(&x.max_eq)?,
                sig_num: x.sig_num,
            };
            Ok(rank)
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(|err| BackendError::RequestParamInvalid(err))?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new()?;

    let txid = cli.update_rank(&main_account, strategy).await?;
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::UpdateStrategy,
        &device.hold_pubkey.unwrap(),
        &device.id,
        &device.brand,
        vec![txid],
    );
    record.insert()?;

    Ok(None::<String>)
}
