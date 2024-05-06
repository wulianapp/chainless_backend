use std::num::ParseIntError;

use actix_web::HttpRequest;

use blockchain::{coin::Coin, fees_call::FeesCall, multi_sig::{MultiSig, MultiSigRank, StrategyData}, ContractClient};
use models::{
    account_manager::{UserFilter, UserInfoView},
    device_info::{DeviceInfoFilter, DeviceInfoView},
    secret_store::{SecretFilter, SecretStoreView},
    PsqlOp,
};
use tracing::{debug, info, warn};

use crate::wallet::{EstimateTransferFeeResponse, SecretType};
use crate::{utils::token_auth, wallet::EstimateTransferFeeRequest};
use common::{data_structures::CoinType, error_code::{BackendError::ChainError, WalletError}, utils::math::coin_amount::{display2raw, raw2display}};
use common::{
    data_structures::secret_store::SecretStore,
    error_code::{AccountManagerError, BackendError, BackendRes},
    utils::math::*
};
use serde::{Deserialize, Serialize};


use super::{get_fees_priority, MIN_BASE_FEE};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: EstimateTransferFeeRequest,
) -> BackendRes<EstimateTransferFeeResponse> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let cli = blockchain::ContractClient::<MultiSig>::new()?;
    let main_account = super::get_main_account(user_id)?;
    let EstimateTransferFeeRequest { coin, amount } = request_data;
    let coin: CoinType = coin.parse()
    .map_err(
        |_|  BackendError::RequestParamInvalid(format!("{} not support",coin))
    )?;
    let amount = display2raw(&amount)?;

    let fee_coins = super::get_fees_priority(&main_account).await?.ok_or(BackendError::InternalError("not set fees priority".to_string()))?;
    let transfer_value = super::get_value(&coin, amount).await;
    //todo: config max_value
    let fee_value = if transfer_value >= 20_000u128 * BASE_DECIMAL {
        transfer_value / 1000 + MIN_BASE_FEE
    }else{
        20u128 * BASE_DECIMAL / 1000 + MIN_BASE_FEE
    };
    info!("coin: {} ,transfer_value: {},fee_value: {}",coin,raw2display(transfer_value),raw2display(fee_value));

    //todo:
    let mut estimate_res = Default::default();
    for (index,fee_coin)  in fee_coins.into_iter().enumerate() {
        let coin_cli: ContractClient<Coin> = ContractClient::<Coin>::new(fee_coin.clone())?;
        let balance = coin_cli.get_balance(&main_account).await?;
        if balance.is_none(){
            continue;
        }

        let mut balance = balance.unwrap().parse().map_err(|e:ParseIntError| e.to_string())?;
        let freezn_amount = super::get_freezn_amount(&main_account, &fee_coin);
        balance = balance - freezn_amount;


        if fee_coin == coin{
            if amount >= balance {
                Err(WalletError::InsufficientAvailableBalance)?;
            }else {
                balance = balance - amount
            }
        }

        let balance_value = super::get_value(&fee_coin, balance).await;
        info!("coin: {} ,fee_value: {},balance_value: {}",fee_coin,raw2display(fee_value),raw2display(balance_value));

        if balance_value  > fee_value{
            //fixme: repeat code
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let (base_amount, quote_amount) = fees_cli.get_coin_price(&fee_coin).await.unwrap();
        let fee_coin_amount = fee_value * base_amount / quote_amount;
            estimate_res = EstimateTransferFeeResponse {
                coin: fee_coin,
                amount: raw2display(fee_coin_amount),
                balance_enough: true,
            };
            break;
        }

        if index == 0 {
            //fixme: repeat code
        let fees_cli = ContractClient::<FeesCall>::new().unwrap();
        let (base_amount, quote_amount) = fees_cli.get_coin_price(&fee_coin).await.unwrap();
        let fee_coin_amount = fee_value * base_amount / quote_amount;
            estimate_res = EstimateTransferFeeResponse {
                coin: fee_coin,
                amount: raw2display(fee_coin_amount),
                balance_enough: false,
            }
        }
    }
    Ok(Some(estimate_res))
}
