#![feature(async_closure)]
//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};

use serde::{Deserialize, Serialize};
use tracing::{debug, Level};

//use captcha::{ContactType, VerificationCode};

use crate::utils::respond::gen_extra_respond;


/**
 * @api {post} /bridge/preWithdraw 主钱包发起提现跨链的预交易
 * @apiVersion 0.0.1
 * @apiName PreWithdraw
 * @apiGroup Bridge
 * @apiBody {String=BTC,ETH,USDT,USDC,DW20} coin      币种名字
 * @apiBody {Number} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
 * @apiBody {String} [captcha]      如果是无需从设备签名的交易，则需要验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
            "coin":"dw20",
            "amount": 123,
            "expireAt": 1708015513000
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/preSendMoneyToBridge
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreWithdrawRequest {
    coin: String,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
    captcha: Option<String>
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/preWithdraw")]
async fn pre_withdraw(
    request: HttpRequest,
    request_data: web::Json<PreWithdrawRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::pre_withdraw::req(request, request_data.0).await)
}


/**
 * @api {post} /bridge/bindEthAddr 在chainless绑定eth地址
 * @apiVersion 0.0.1
 * @apiName BindEthAddr
 * @apiGroup Bridge
 * @apiBody {String} eth_addr   以太坊地址
 * @apiBody {String} user_eth_sig   以太坊私钥签名结果 
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/bridge/commitWithdraw -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {string=0,1,2,2002,2003,2004,2005} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError,RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/bridge/bindEthAddr
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BindEthAddrRequest {
    eth_addr: String,
    user_eth_sig: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/bindEthAddr")]
async fn bind_eth_addr(
    req: HttpRequest,
    request_data: web::Json<BindEthAddrRequest>
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::bind_eth_addr::req(req,request_data.into_inner()).await)
}


/**
 * @api {post} /bridge/genBindEthAddrSig  生成ETH地址绑定信息签名
 * @apiVersion 0.0.1
 * @apiName GenBindEthAddrSig
 * @apiGroup Bridge
 * @apiBody {String="SetSecurity","ResetLoginPassword","PreSendMoney","PreSendMoneyToSub","ServantSwitchMaster","NewcomerSwitchMaster"} kind 验证码类型，测试网生成的验证码为000000
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/bridge/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {string=0,1,2,2002,2003,2004,2005} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError,RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/bridge/AccountManager
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenBindEthAddrSigRequest {
    eth_addr:String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/genBindEthAddrSig")]
async fn gen_bind_eth_addr_sig(request: HttpRequest,
    request_data: web::Json<GenBindEthAddrSigRequest>) 
-> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::gen_bind_eth_addr_sig::req(request,request_data.into_inner()).await)
}

/**
 * @api {post} /bridge/genDepositSig  生成跨链提现
 * @apiVersion 0.0.1
 * @apiName GenDepositSig
 * @apiGroup Bridge
 * @apiBody {String="SetSecurity","ResetLoginPassword","PreSendMoney","PreSendMoneyToSub","ServantSwitchMaster","NewcomerSwitchMaster"} kind 验证码类型，测试网生成的验证码为000000
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {string=0,1,2,2002,2003,2004,2005} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError,RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/bridge/AccountManager
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenDepositSigRequest {
    coin:String,
    amount:u128,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/genDepositSig")]
async fn gen_deposit_sig(request: HttpRequest,
    request_data: web::Json<GenDepositSigRequest>) 
-> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::gen_deposit_sig::req(request,request_data.into_inner()).await)
}

/**
 * @api {get} /bridge/getBindedEthAddr 获取用户绑定的eth地址
 * @apiVersion 0.0.1
 * @apiName GetBindedEthAddr
 * @apiGroup Bridge
 * @apiBody {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/userInfo"
 * @apiSuccess {string=0,1,} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {object} data                当前绑定的eth地址
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/userInfo
 */

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/bridge/getBindedEthAddr")]
async fn get_binded_eth_addr(request: HttpRequest) -> impl Responder {
    //debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_binded_eth_addr::req(request).await)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(bind_eth_addr)
        .service(gen_bind_eth_addr_sig)
        .service(gen_deposit_sig)
        .service(get_binded_eth_addr)
        .service(pre_withdraw);
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::utils::respond::BackendRespond;
    use crate::utils::api_test::*;

    use super::*;
    use std::default::Default;
    use std::env;

    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;

    use actix_web::{body::MessageBody as _, test, App};

    use blockchain::ContractClient;
    use common::data_structures::device_info::DeviceInfo;
    use common::data_structures::KeyRole;
    use models::coin_transfer::CoinTxView;
    use models::{account_manager, secret_store, PsqlOp};
    use serde_json::json;

    use actix_web::Error;
    use blockchain::multi_sig::{StrategyData};
    use common::encrypt::{ed25519_key_gen};
    use blockchain::multi_sig::{CoinTx, MultiSig};
    use common::data_structures::account_manager::UserInfo;
    use common::data_structures::secret_store::SecretStore;
    use common::data_structures::wallet::{AccountMessage, CoinTxStatus, TxType};
    use common::utils::math;
    use models::secret_store::SecretStoreView;
    // use log::{info, LevelFilter,debug,error};
    use common::data_structures::wallet::CoinType;
    use models::account_manager::UserInfoView;
    use tracing::{debug, error, info};
    use crate::wallet::handlers::get_tx::CoinTxViewTmp2;
    use std::collections::HashMap;
    use crate::wallet::handlers::balance_list::AccountBalance;

    #[actix_web::test]
    async fn test_bridge_deposit() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,
            _sender_servant,
            mut sender_newcommer,
            _receiver) 
        = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_login!(service, sender_newcommer);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;


        /***     
        test_get_captcha_with_token!(service,sender_newcommer,"NewcomerSwitchMaster");
        let gen_res = test_gen_newcommer_switch_master!(service,sender_newcommer);

        let add_key_sig = common::encrypt::ed25519_sign_hex(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &gen_res.as_ref().unwrap().add_key_txid,
        ).unwrap();

        let delete_key_sig = common::encrypt::ed25519_sign_hex(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &gen_res.as_ref().unwrap().delete_key_txid,
        ).unwrap();
        test_commit_newcommer_switch_master!(service,sender_newcommer,gen_res,add_key_sig,delete_key_sig);
        */
    }
}