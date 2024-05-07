use std::str::FromStr;

use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig};
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, TxType};
use common::data_structures::CoinType;

use common::data_structures::KeyRole2;
use common::encrypt::bs58_to_hex;
use common::utils::math::coin_amount::display2raw;
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoView};

use crate::utils::token_auth;
use crate::wallet::SubSendToMainRequest;
use blockchain::multi_sig::AccountSignInfo;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxFilter, CoinTxUpdater, CoinTxView};
use models::PsqlOp;

pub async fn req(
    req: HttpRequest,
    request_data: web::Json<SubSendToMainRequest>,
) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let (user, current_strategy, device) = super::get_session_state(user_id, &device_id).await?;
    let main_account = user.main_account;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    //todo: check must be main device
    let SubSendToMainRequest {
        sub_sig,
        subaccount_id,
        coin,
        amount,
    } = request_data.0;
    let amount = display2raw(&amount).map_err(|err| BackendError::RequestParamInvalid(err))?;

    //from必须是用户的子账户
    let cli = ContractClient::<MultiSig>::new()?;

    let sub_sig = AccountSignInfo::new(&subaccount_id, &sub_sig);
    let coin_type: CoinType = coin.parse().map_err(|e| BackendError::RequestParamInvalid("coin not support".to_string()))?;

    models::general::transaction_begin()?;
    let tx_id = cli
        .internal_transfer_sub_to_main(&main_account, sub_sig.clone(), coin_type.clone(), amount)
        .await?;    
    let mut coin_info = CoinTxView::new_with_specified(
        coin_type,
        sub_sig.account_id,
        main_account,
        amount,
        "".to_string(),
        None,
        u64::MAX,
        CoinSendStage::SenderReconfirmed,
    );
    coin_info.transaction.tx_type = TxType::SubToMain;
    coin_info.transaction.tx_id = Some(tx_id.clone());
    coin_info.insert()?;
    models::general::transaction_commit()?;
    Ok(Some(tx_id))
}
