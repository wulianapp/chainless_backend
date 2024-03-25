use std::str::FromStr;

use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, CoinType, TxType};
use common::data_structures::KeyRole2;
use models::device_info::{DeviceInfoFilter, DeviceInfoView};
use tracing::debug;

use crate::utils::captcha::{Captcha, Usage};
use crate::utils::token_auth;
use common::error_code::{AccountManagerError, BackendError, BackendRes, WalletError};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView};

use models::coin_transfer::{get_next_tx_index, CoinTxView};
use models::PsqlOp;

use crate::wallet::PreSendMoneyRequest;

pub(crate) async fn req(req: HttpRequest, request_data: PreSendMoneyRequest) -> BackendRes<(u32,String)> {
    //todo: allow master only
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let main_account = super::get_main_account(user_id)?;
    let user_email = super::get_email(user_id)?;

    let PreSendMoneyRequest {
        to,
        coin,
        amount,
        expire_at,
        memo,
        is_forced,
        captcha
    } = request_data;
    let from  = main_account.clone();
    let coin_type = CoinType::from_str(&coin).unwrap();

    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }
    let available_balance = super::get_available_amount(&from,&coin_type).await?;
    let available_balance = available_balance.unwrap_or(0);
    if amount > available_balance {
        Err(WalletError::InsufficientAvailableBalance)?;
    }
    //如果本身是单签，则状态直接变成SenderSigCompleted
    let cli = ContractClient::<MultiSig>::new();
    let strategy = cli
        .get_strategy(&main_account)
        .await?
        .ok_or(WalletError::SenderNotFound)?;
    if let Some(sub_conf) = strategy.sub_confs.get(&to){
        debug!("to[{}] is subaccount of from[{}]",to,from);
        let coin_price = 1;
        let balance_value = cli.get_total_value(&to).await;
        if  amount * coin_price + balance_value > sub_conf.hold_value_limit {
            Err(WalletError::ExceedSubAccountHoldLimit)?;
        }
    }

    let gen_tx_with_status = |status: CoinTxStatus|{
        let coin_tx_raw = cli
            .gen_send_money_info(&from, &to, coin_type.clone(), amount, expire_at)
            .unwrap();
        CoinTxView::new_with_specified(
            coin_type.clone(),
            from.clone(),
            to.clone(),
            amount,
            coin_tx_raw,
            memo,
            expire_at,
            status,
        )
    };

    //交易收到是否从设备为空、是否强制转账、是否是转子账户三种因素的影响
    //将转子账户的接口单独剥离出去
    //let servant_is_empty =  strategy.servant_pubkeys.is_empty();
    let to_is_sub = strategy.sub_confs.get(&to).is_some();
    if to_is_sub {
        Err(BackendError::InternalError("to cannt be subaccount".to_string()))?;
    }

     let need_sig_num = super::get_servant_need(
        &strategy.multi_sig_ranks,
        &coin_type,
        amount
    ).await; 

    //没有从公钥且强制转账的话，直接返回待签名数据
    if need_sig_num == 0 && is_forced{
        debug!("_0000_");
        if captcha.is_none(){
            Err(BackendError::InternalError("For single tx,need captcha".to_string()))?;
        } 
        Captcha::check_user_code(&user_email, &captcha.unwrap(), Usage::PreSendMoney)?;
        let mut coin_info = gen_tx_with_status( CoinTxStatus::ReceiverApproved);
        let next_tx_index = get_next_tx_index()?;
        let (tx_id, chain_tx_raw) = cli
        .gen_send_money_raw(
            next_tx_index as u64,
            vec![],
            &from,
            &to,
            coin_type,
            amount,
            expire_at,
        )
        .await
        .unwrap()
        .unwrap();
        coin_info.transaction.chain_tx_raw = Some(chain_tx_raw);
        coin_info.transaction.tx_type = TxType::Forced;
        coin_info.insert()?;
        Ok(Some((next_tx_index,tx_id)))
    }else if need_sig_num == 0  && !is_forced{
        debug!("_0001_");
        if captcha.is_none(){
            Err(BackendError::InternalError("For single tx,need captcha".to_string()))?;
        } 
        Captcha::check_user_code(&user_email, &captcha.unwrap(), Usage::PreSendMoney)?;
        let coin_info = gen_tx_with_status(CoinTxStatus::SenderSigCompleted);
        coin_info.insert()?;
        Ok(None)
    }else if need_sig_num != 0  && is_forced{
        debug!("_0002_");
        if captcha.is_some(){
            Err(BackendError::InternalError("For sulti-sig tx,need not captcha".to_string()))?;
        }
        let mut coin_info = gen_tx_with_status(CoinTxStatus::Created);
        coin_info.transaction.tx_type = TxType::Forced;
        coin_info.insert()?;
        Ok(None)
    }else if need_sig_num != 0  && !is_forced{
        debug!("_0003_");
        if captcha.is_some(){
            Err(BackendError::InternalError("For multi-sig tx,need not  captcha".to_string()))?;
        }
        let coin_info = gen_tx_with_status(CoinTxStatus::Created);
        coin_info.insert()?;
        Ok(None)
    }else {
        unreachable!("all case is considered")
    }
}
