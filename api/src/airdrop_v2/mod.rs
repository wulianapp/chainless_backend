//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};
use handlers::get_grade::GetGradeRequest;
use tracing::debug;

use handlers::bind_btc_address::BindBtcAddressRequest;
use handlers::change_invite_code::ChangeInviteCodeRequest;
use handlers::change_predecessor::ChangePredecessorRequest;
use handlers::new_btc_deposit::NewBtcDepositRequest;
use crate::utils::respond::gen_extra_respond;
use crate::utils::respond::get_lang;
use crate::utils::respond::get_trace_id;

/**
* @api {get} /airdrop_v2/status 获取后台存储中的用户空投状态
* @apiVersion 0.0.2
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
* @apiSuccess {String} [data.ref_btc_address]         btc大号地址
* @apiSuccess {String=NotBind,PendingCalculate,Calculated,Reconfirmed} data.btc_grade_status      btc地址的评级状态
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/status
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/airdrop_v2/status")]
async fn status(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::status::req(req).await)
}


/**
* @api {get} /airdrop_v2/getGrade 查询地址对应的等级
* @apiVersion 0.0.2
* @apiName GetGrade
* @apiGroup Airdrop
* @apiQuery {String}  btcAddress    btc地址
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/getStrategy
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3011} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {Number} data                          地址等级.
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/status
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/airdrop_v2/getGrade")]
async fn get_grade(req: HttpRequest,request_data: web::Query<GetGradeRequest>) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(get_lang(&req), handlers::get_grade::req(req,request_data.into_inner()).await)
}

/**
 * @api {post} /airdrop_v2/bindBtcAddress 绑定btc地址
 * @apiVersion 0.0.2
 * @apiName BindBtcAddress
 * @apiGroup Airdrop
 * @apiBody {String} btcAddress   btc地址
 * @apiBody {String} sig   btc私钥对user_id的字符串的签名结果
 * @apiBody {String=Directly,Indirectly} way   地址对应的绑定方式,前者对应导入私钥，后者对应创建新钱包
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
* @apiSuccess {Number}   [data]            Directly返回等级，Indirectly返回null
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/bindBtcAddress
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/bindBtcAddress")]
async fn bind_btc_address(
    req: HttpRequest,
    request_data: web::Json<BindBtcAddressRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::bind_btc_address::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /airdrop_v2/changeInviteCode 修改邀请码
 * @apiVersion 0.0.2
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
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/changeInviteCode
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/changeInviteCode")]
async fn change_invite_code(
    req: HttpRequest,
    request_data: web::Json<ChangeInviteCodeRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::change_invite_code::req(req, request_data.into_inner()).await,
    )
}


/**
 * @api {post} /airdrop_v2/resetStatus 重置空投状态
 * @apiVersion 0.0.2
 * @apiName ResetStatus
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
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/resetStatus
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/resetStatus")]
async fn reset_status(
    req: HttpRequest
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::reset_status::req(req).await
    )
}

/**
 * @api {post} /airdrop_v2/changePredecessor 修改上级
 * @apiVersion 0.0.2
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
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/changePredecessor
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/changePredecessor")]
async fn change_predecessor(
    req: HttpRequest,
    request_data: web::Json<ChangePredecessorRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::change_predecessor::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /airdrop_v2/claimCly 登记cly空投
 * @apiVersion 0.0.2
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
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/claimCly
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/claimCly")]
async fn claim_cly(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::claim_cly::req(req).await)
}

/**
 * @api {post} /airdrop_v2/claimDw20 登记Dw20空投
 * @apiVersion 0.0.2
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
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/claimDw20
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/claimDw20")]
async fn claim_dw20(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::claim_dw20::req(req).await)
}

/**
 * @api {post} /airdrop_v2/newBtcDeposit （内部调用无权限限制）注入新的btc的符合规则的充值
 * @apiVersion 0.0.2
 * @apiName NewBtcDeposit
 * @apiGroup Airdrop
 * @apiBody {Object[]}     utxoArray                 utxo组
 * @apiBody {String}     utxoArray.sender               发送方btc地址
 * @apiBody {String}     utxoArray.recipient               接收方btc地址
 * @apiBody {Number}     utxoArray.value                  value
 * @apiBody {Number}     utxoArray.blockheight           blockheight
 * @apiBody {Number}     utxoArray.blocktime             blocktime
 * @apiBody {String}     utxoArray.txid               交易hash
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066
   -d ' {
             "servantPubkey": "123",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007,3008,3011} status_code         状态码.
* @apiSuccess {String}    msg              错误信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/airdrop_v2/newBtcDeposit
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/airdrop_v2/newBtcDeposit")]
async fn new_btc_deposit(
    req: HttpRequest,
    request_data: web::Json<NewBtcDepositRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
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
        .service(get_grade)
        .service(reset_status)
        .service(new_btc_deposit);
}

#[cfg(test)]
mod tests {
    use crate::utils::api_test::*;
    use crate::utils::respond::BackendRespond;
    use crate::*;

    use actix_web::body::MessageBody;

    use actix_web::http::header;

    use actix_web::test;

    use common::btc_crypto::{self, calculate_p2tr_address, new_secret_key};
    use serde_json::json;

    // use log::{info, LevelFilter,debug,error};
    use super::handlers::status::AirdropStatusResponse;

    #[actix_web::test]
    async fn test_airdrop_braced() {
        let app = init().await;
        let service = actix_web::test::init_service(app).await;
        let (mut sender_master, _sender_servant, _sender_newcommer, _receiver) =
            gen_some_accounts_with_new_key();

        let (btc_prikey, btc_pubkey) = new_secret_key().unwrap();
        let btc_addr = calculate_p2tr_address(&btc_pubkey).unwrap();
        println!("btc_address_{}", btc_addr);
        let btc_addr = "bcrt1ptnmsjes32gz4nelu6m9ghm8lg2qp536w9cs7knn9the7g7rz6gpqm6pjus";
        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        //tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info1 {:#?}", status_info);

        let signature = btc_crypto::sign(&btc_prikey, &status_info.user_id.to_string()).unwrap();
        test_bind_btc_address!(service, sender_master, btc_addr, signature);

        //test_new_btc_deposit!(service, sender_master);
        test_change_invite_code!(service, sender_master);

        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info2 {:#?}", status_info);

        test_claim_dw20!(service, sender_master);
        test_claim_cly!(service, sender_master);
        test_change_predecessor!(service, sender_master);

        let status_info = test_airdrop_status!(service, sender_master).unwrap();
        println!("status_info3 {:#?}", status_info);
    }
}
