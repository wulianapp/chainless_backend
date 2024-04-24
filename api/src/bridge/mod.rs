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
 * @apiBody {String} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
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
* @apiSuccess {String=0,1} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/preSendMoneyToBridge
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreWithdrawRequest {
    coin: String,
    amount: String,
    expire_at: u64,
    memo: Option<String>,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/preWithdraw")]
async fn pre_withdraw(
    request: HttpRequest,
    request_data: web::Json<PreWithdrawRequest>,
) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
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
 * @apiSuccess {String=0,1,2,2002,2003,2004,2005} status_code         状态码.
 * @apiSuccess {String=RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {String} data                nothing.
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
    request_data: web::Json<BindEthAddrRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::bind_eth_addr::req(req, request_data.into_inner()).await)
}

/**
 * @api {post} /bridge/genBindEthAddrSig  生成ETH地址绑定信息签名
 * @apiVersion 0.0.1
 * @apiName GenBindEthAddrSig
 * @apiGroup Bridge
 * @apiBody {String} ethAddr 以太坊地址
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/bridge/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {String=0,1,2,2002,2003,2004,2005} status_code         状态码.
 * @apiSuccess {String=RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {String} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/bridge/AccountManager
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenBindEthAddrSigRequest {
    eth_addr: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/genBindEthAddrSig")]
async fn gen_bind_eth_addr_sig(
    request: HttpRequest,
    request_data: web::Json<GenBindEthAddrSigRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        handlers::gen_bind_eth_addr_sig::req(request, request_data.into_inner()).await,
    )
}

/**
* @api {post} /bridge/genDepositSig  生成跨链充值
* @apiVersion 0.0.1
* @apiName GenDepositSig
* @apiGroup Bridge
* @apiBody {String="BTC","ETH","USDT","USDC","DW20"} coin 币种类型
* @apiBody {String} amount 提现数量
* @apiBody {String} ethDepositor 充值方的eth地址
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken -H "Content-Type: application/json" -d
*  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
* @apiSuccess {String=0,1,2,2002,2003,2004,2005} status_code         状态码.
* @apiSuccess {String=RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
* @apiSuccess {object} data                  签名和过期时间戳.
* @apiSuccess {String} data.sig                签名.
* @apiSuccess {Number} data.deadline                过期时间戳.
* @apiSuccess {Number} data.cid                签名随机值cid

* @apiSampleRequest http://120.232.251.101:8066/bridge/AccountManager
*/
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenDepositSigRequest {
    coin: String,
    amount: String,
    eth_depositor: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/bridge/genDepositSig")]
async fn gen_deposit_sig(
    request: HttpRequest,
    request_data: web::Json<GenDepositSigRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::gen_deposit_sig::req(request, request_data.into_inner()).await)
}

/**
 * @api {get} /bridge/getBindedEthAddr 获取用户绑定的eth地址
 * @apiVersion 0.0.1
 * @apiName GetBindedEthAddr
 * @apiGroup Bridge
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/bridge/getBindedEthAddr"
 * @apiSuccess {String=0,1,} status_code         状态码.
 * @apiSuccess {String} msg 状态信息
 * @apiSuccess {String=null} data                当前绑定的eth地址，
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
    use crate::utils::api_test::*;
    use crate::utils::respond::BackendRespond;
    use crate::*;

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
    use blockchain::multi_sig::StrategyData;
    use blockchain::multi_sig::{CoinTx, MultiSig};
    use common::data_structures::account_manager::UserInfo;
    use common::data_structures::secret_store::SecretStore;
    use common::encrypt::ed25519_key_gen;
    use common::utils::math;
    use models::secret_store::SecretStoreView;
    // use log::{info, LevelFilter,debug,error};
    use crate::account_manager::handlers::user_info::UserInfoTmp;
    use common::data_structures::CoinType;
    use models::account_manager::UserInfoView;
    use std::collections::HashMap;
    use tracing::{debug, error, info};

    #[actix_web::test]
    async fn test_bind_eth_addr() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master, _sender_servant, mut sender_newcommer, _receiver) =
            gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_login!(service, sender_newcommer);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let user_info = test_user_info!(service, sender_master).unwrap();
        let bridge_cli = ContractClient::<blockchain::bridge_on_near::Bridge>::new().unwrap();
        let sig = bridge_cli
            .sign_bind_info(
                &user_info.main_account,
                "cb5afaa026d3de65de0ddcfb1a464be8960e334a",
            )
            .await;
        println!("sign_bind sig {} ", sig);

        //todo: sig on imtoken and verify on server
        let bind_res = bridge_cli
            .bind_eth_addr(
                &user_info.main_account,
                "cb5afaa026d3de65de0ddcfb1a464be8960e334a",
                &sig,
            )
            .await
            .unwrap();
        println!("bind_res {} ", bind_res);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let current_bind_eth_addr = bridge_cli
            .get_binded_eth_addr(&user_info.main_account)
            .await
            .unwrap();
        println!("bind_res {:?} ", current_bind_eth_addr);
    }
}
