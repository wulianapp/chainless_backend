#![feature(async_closure)]
//! account manager http service
pub mod handlers;

use std::future::IntoFuture;

use actix_web::{get, post, web, HttpRequest, Responder};

use blockchain::bridge_on_near;
use blockchain::bridge_on_near::Status;
use common::data_structures::{
    bridge::{DepositStatus, WithdrawStatus},
    CoinType,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, Level};

//use captcha::{ContactType, VerificationCode};

use crate::airdrop::handlers::bind_btc_address::BindBtcAddressRequest;
use crate::airdrop::handlers::change_invite_code::ChangeInviteCodeRequest;
use crate::airdrop::handlers::change_predecessor::ChangePredecessorRequest;
use crate::airdrop::handlers::new_btc_deposit::NewBtcDepositRequest;
use crate::utils::respond::gen_extra_respond;
use crate::utils::respond::get_lang;
use common::log::generate_trace_id;

/**
* @api {get} /airdrop/status 获取后台存储中的用户空投状态
* @apiVersion 0.0.1
* @apiName Status
* @apiGroup Airdrop
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/getStrategy
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3011} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {Object} data                          空投状态详情.
* @apiSuccess {String} data.user_id       用户id
* @apiSuccess {String} [data.account_id]    钱包id
* @apiSuccess {String} data.invite_code   邀请码
* @apiSuccess {String} data.predecessor_user_id      上级用户id
* @apiSuccess {String} data.predecessor_account_id   上级钱包id
* @apiSuccess {String} [data.btc_address]       绑定的btc钱包地址
* @apiSuccess {Number} [data.btc_level]         btc地址对应的等级
* @apiSuccess {String} [data.cly_claimed]       cly的空投数量
* @apiSuccess {String} [data.dw20_claimed]      dw20的空投数量
* @apiSampleRequest http://120.232.251.101:8066/airdrop/status
*/

#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[get("/airdrop/status")]
async fn status(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::status::req(req).await)
}

/**
 * @api {post} /airdrop/bindBtcAddress 绑定btc地址
 * @apiVersion 0.0.1
 * @apiName BindBtcAddress
 * @apiGroup Airdrop
 * @apiBody {String} btcAddress   btc地址
 * @apiBody {String} sig   btc私钥对字符串 ChainlessAirdrop 签名结果
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/bindBtcAddress
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/bindBtcAddress")]
async fn bind_btc_address(
    req: HttpRequest,
    req_data: web::Json<BindBtcAddressRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::bind_btc_address::req(req, req_data.into_inner()).await,
    )
}

/**
 * @api {post} /airdrop/changeInviteCode 修改邀请码
 * @apiVersion 0.0.1
 * @apiName ChangeInviteCode
 * @apiGroup Airdrop
 * @apiBody {String} code   新的邀请码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/changeInviteCode
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/changeInviteCode")]
async fn change_invite_code(
    req: HttpRequest,
    req_data: web::Json<ChangeInviteCodeRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::change_invite_code::req(req, req_data.into_inner()).await,
    )
}

/**
 * @api {post} /airdrop/changePredecessor 修改上级
 * @apiVersion 0.0.1
 * @apiName ChangePredecessor
 * @apiGroup Airdrop
 * @apiBody {String} predecessorInviteCode   上级钱包的邀请码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/changePredecessor
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/changePredecessor")]
async fn change_predecessor(
    req: HttpRequest,
    req_data: web::Json<ChangePredecessorRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::change_predecessor::req(req, req_data.into_inner()).await,
    )
}

/**
 * @api {post} /airdrop/claimCly 登记cly空投
 * @apiVersion 0.0.1
 * @apiName ClaimCly
 * @apiGroup Airdrop
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/claimCly
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/claimCly")]
async fn claim_cly(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::claim_cly::req(req).await)
}

/**
 * @api {post} /airdrop/claimDw20 登记Dw20空投
 * @apiVersion 0.0.1
 * @apiName ClaimDw20
 * @apiGroup Airdrop
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/claimDw20
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/claimDw20")]
async fn claim_dw20(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::claim_dw20::req(req).await)
}

/**
 * @api {post} /airdrop/newBtcDeposit （内部调用）注入新的btc的符合规则的充值
 * @apiVersion 0.0.1
 * @apiName NewBtcDeposit
 * @apiGroup Airdrop
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop/newBtcDeposit
*/
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[post("/airdrop/newBtcDeposit")]
async fn new_btc_deposit(
    req: HttpRequest,
    request_data: web::Json<NewBtcDepositRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::new_btc_deposit::req(req, request_data.into_inner()).await,
    )
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(status)
        .service(bind_btc_address)
        .service(change_invite_code)
        .service(change_predecessor)
        .service(claim_cly)
        .service(claim_dw20)
        .service(new_btc_deposit);
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
    use models::coin_transfer::CoinTxEntity;
    use models::{account_manager, secret_store, PsqlOp};
    use serde_json::json;

    use actix_web::Error;
    use blockchain::multi_sig::StrategyData;
    use blockchain::multi_sig::{CoinTx, MultiSig};
    use common::data_structures::account_manager::UserInfo;
    use common::data_structures::secret_store::SecretStore;
    use common::encrypt::ed25519_key_gen;
    use common::utils::math;
    use models::secret_store::SecretStoreEntity;
    // use log::{info, LevelFilter,debug,error};
    use super::handlers::status::AirdropStatusResponse;
    use crate::account_manager::handlers::user_info::UserInfoResponse;
    use actix_web::http::header::HeaderName;
    use actix_web::http::header::HeaderValue;
    use common::data_structures::CoinType;
    use models::account_manager::UserInfoEntity;
    use std::collections::HashMap;
    use tracing::{debug, error, info};

    #[actix_web::test]
    async fn test_airdrop_braced() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master, _sender_servant, _sender_newcommer, _receiver) =
            gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        //tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info1 {:#?}", status_info);

        test_bind_btc_address!(service, sender_master);
        test_new_btc_deposit!(service, sender_master);
        test_change_invite_code!(service, sender_master);
        //test_change_predecessor!(service,sender_master);

        //test_change_invite_code!(service,sender_master);1
        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info2 {:#?}", status_info);

        test_claim_dw20!(service, sender_master);
        test_claim_cly!(service, sender_master);
        test_change_predecessor!(service, sender_master);

        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info3 {:#?}", status_info);
    }
}
