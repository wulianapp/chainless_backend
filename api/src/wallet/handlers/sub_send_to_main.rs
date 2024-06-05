use std::str::FromStr;

use actix_web::{web, HttpRequest};

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::coin_transaction::{CoinSendStage, SubToMainTx, TxType};
use common::data_structures::CoinType;

use common::data_structures::KeyRole2;
use common::encrypt::{bs58_to_hex, ed25519_verify_hex, ed25519_verify_raw};
use common::utils::math::coin_amount::display2raw;
use models::account_manager::{UserFilter, UserInfoEntity};
use models::device_info::{DeviceInfoEntity, DeviceInfoFilter};
use models::general::get_pg_pool_connect;
use tracing::error;

use crate::utils::token_auth;
use blockchain::multi_sig::AccountSignInfo;
use common::error_code::{BackendError, BackendRes, WalletError};
use models::coin_transfer::{CoinTxEntity, CoinTxFilter, CoinTxUpdater};
use models::PsqlOp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubSendToMainRequest {
    sub_sig: String,
    subaccount_id: String,
    coin: String,
    amount: String,
}

pub async fn req(req: HttpRequest, request_data: SubSendToMainRequest) -> BackendRes<String> {
    //todo:check user_id if valid
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let mut db_cli = get_pg_pool_connect().await?;
    let (user, current_strategy, device) =
        super::get_session_state(user_id, &device_id, &mut db_cli).await?;
    let main_account = user.main_account.clone().unwrap();
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;

    //todo: check must be main device
    let SubSendToMainRequest {
        sub_sig,
        subaccount_id,
        coin: coin_id,
        amount,
    } = request_data;
    let amount = display2raw(&amount).map_err(|_e|WalletError::UnSupportedPrecision)?;

    let coin_type: CoinType = coin_id
        .parse()
        .map_err(|_e| BackendError::RequestParamInvalid("coin not support".to_string()))?;

    let cli = ContractClient::<MultiSig>::new_update_cli().await?;

    // check sig
    let tx = SubToMainTx {
        coin_id: coin_type.to_string(),
        amount,
    };
    let sign_data = hex::encode(serde_json::to_string(&tx).unwrap().as_bytes());
    let key = cli.get_master_pubkey_list(&subaccount_id).await?;
    if key.len() != 1 {
        return Err(BackendError::InternalError("".to_string()))?;
    } else if !ed25519_verify_hex(&sign_data, &key[0], &sub_sig)? {
        Err(BackendError::SigVerifyFailed)?;
    };

    let available_balance =
        super::get_available_amount(&subaccount_id, &coin_type, &mut db_cli).await?;
    let available_balance = available_balance.unwrap_or(0);

    if amount > available_balance {
        error!(
            "{},  {}(amount)  big_than1 {}(available_balance) ",
            coin_type, amount, available_balance
        );
        Err(WalletError::InsufficientAvailableBalance)?;
    }

    //from必须是用户的子账户
    let sub_sig = AccountSignInfo::new(&subaccount_id, &sub_sig);

    let tx_id = cli
        .internal_transfer_sub_to_main(&main_account, sub_sig.clone(), coin_type.clone(), amount)
        .await?;

    let mut coin_info = CoinTxEntity::new_with_specified(
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
    coin_info.insert(&mut db_cli).await?;
    Ok(Some(tx_id))
}
