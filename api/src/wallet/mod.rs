//! account manager http service

pub mod handlers;
mod transaction;

use actix_web::web::service;
use actix_web::{get, post, web, HttpRequest, Responder};

use common::data_structures::secret_store::SecretStore;
use common::data_structures::wallet::CoinType;
use serde::{Deserialize, Serialize};

use crate::utils::respond::gen_extra_respond;
//use crate::transaction::{get_all_message, get_user_message, insert_new_message, MessageType, update_message_status};

use crate::account_manager::{
    contact_is_used, login, register_by_email, register_by_phone, reset_password,
};
use tracing::{debug, span, Level};

use self::handlers::balance_list::AccountType;

/**
* @api {get} /wallet/searchMessage 查询待处理的钱包消息
* @apiVersion 0.0.1
* @apiName searchMessage
* @apiGroup Wallet
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/searchMessage
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg                 description of status.
* @apiSuccess {object[]} data                当前需要处理的消息详情.
* @apiSuccess {object} data.NewcomerBecameSevant    新设备成为从设备消息
* @apiSuccess {string} data.NewcomerBecameSevant.pubkey           被分配的servant_pubkey
* @apiSuccess {string} data.NewcomerBecameSevant.state            不用关注
* @apiSuccess {Number} data.NewcomerBecameSevant.user_id          所属用户id
* @apiSuccess {string} data.NewcomerBecameSevant.encrypted_prikey_by_password    安全密码加密私钥的输出
* @apiSuccess {string} data.NewcomerBecameSevant.encrypted_prikey_by_answer      安全问答加密私钥的输出
* @apiSuccess {object[]} data.CoinTx                转账消息
* @apiSuccess {Number} data.CoinTx.tx_index          交易索引号.
* @apiSuccess {object} data.CoinTx.transaction        交易详情.
* @apiSuccess {String} [data.CoinTx.transaction.tx_id]        链上交易id.
* @apiSuccess {String=dw20,cly} data.CoinTx.transaction.coin_type      币种名字
* @apiSuccess {String} data.CoinTx.transaction.from                发起方
* @apiSuccess {String} data.CoinTx.transaction.to                接收方
* @apiSuccess {Number} data.CoinTx.transaction.amount               交易量
* @apiSuccess {String} data.CoinTx.transaction.expireAt             交易截止时间戳
* @apiSuccess {String} [data.CoinTx.transaction.memo]                交易备注
* @apiSuccess {String=Created,SenderSigCompleted,ReceiverApproved,ReceiverRejected,SenderCanceled,SenderReconfirmed} data.CoinTx.transaction.status                交易状态
* @apiSuccess {String}  data.CoinTx.transaction.coin_tx_raw       币种转账的业务原始数据hex
* @apiSuccess {String} [data.CoinTx.transaction.chain_tx_raw]          链上交互的原始数据
* @apiSuccess {String[]} data.CoinTx.transaction.signatures         从设备对业务数据的签名
* @apiSuccess {String=Normal,Forced,ToSub,FromSub} data.CoinTx.transaction.tx_type         从设备对业务数据的签名
* @apiSampleRequest http://120.232.251.101:8066/wallet/searchMessage
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/searchMessage")]
async fn search_message(request: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::search_message::req(request).await)
}

/**
 * @api {get} /wallet/getStrategy 查询钱包的主从签名策略
 * @apiVersion 0.0.1
 * @apiName getStrategy
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/getStrategy
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
 * @apiSuccess {string=0,1} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {Object} data                          策略详情.
 * @apiSuccess {String} data.master_pubkey        主钱包的maser的公钥
 * @apiSuccess {String[]} data.servant_pubkeys    主钱包的servant的公钥组
 * @apiSuccess {Object[]} data.subaccounts        子钱包的配置
 * @apiSuccess {String} data.subaccounts.0        子钱包的公钥组
 * @apiSuccess {number} data.subaccounts.hold_value_limit   子钱包持仓限制
 * @apiSuccess {Object[]} [data.multi_sig_ranks]        转账额度对应签名数的档位.
 * @apiSuccess {Number} data.multi_sig_ranks.min       最小金额.
 * @apiSuccess {Number} data.multi_sig_ranks.max_eq        最大金额.
 * @apiSuccess {Number} data.multi_sig_ranks.sig_num        金额区间需要的最小签名数.
 * @apiSampleRequest http://120.232.251.101:8066/wallet/getStrategy
 */
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getStrategy")]
async fn get_strategy(request: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::get_strategy::req(request).await)
}

/**
 * @api {get} /wallet/getFeesPriority 获取抵扣手续费的币种顺序
 * @apiVersion 0.0.1
 * @apiName GetFeesPriority
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/getStrategy
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
 * @apiSuccess {string=0,1} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {String[]} data                          币种顺序
 * @apiSampleRequest http://120.232.251.101:8066/wallet/getStrategy
 */
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getFeesPriority")]
async fn get_fees_priority(request: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::get_fees_priority::req(request).await)
}

/**
* @api {get} /wallet/getSecret 获取备份密钥信息
* @apiVersion 0.0.1
* @apiName GetSecret
* @apiGroup Wallet
* @apiQuery {String=currentDevice,master,all}  type  想获取的密钥类型
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/getSecret
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {Object[]} data                                    备份加密密钥信息.
* @apiSuccess {String} data.pubkey                             对应的公钥
* @apiSuccess {String} data.state                              不关注
* @apiSuccess {Number} data.user_id                            所属用户ID
* @apiSuccess {String} data.encrypted_prikey_by_password       被安全密码加密后的文本
* @apiSuccess {String} data.encrypted_prikey_by_answer         被安全问答加密后的文本
* @apiSampleRequest http://120.232.251.101:8066/wallet/getSecret
*/

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SecretType {
    CurrentDevice,
    Master,
    All,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetSecretRequest {
    pub r#type: SecretType,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getSecret")]
async fn get_secret(
    request: HttpRequest,
    request_data: web::Query<GetSecretRequest>,
) -> impl Responder {
    debug!("req_params:: {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_secret::req(request, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/preSendMoney 主钱包发起预交易
 * @apiVersion 0.0.1
 * @apiName preSendMoney
 * @apiGroup Wallet
 * @apiBody {String} from    发起方多签钱包ID
 * @apiBody {String} to      收款方ID（可以是手机、邮箱、钱包id）
 * @apiBody {String=BTC,ETH,USDT,USDC,DW20,CLY} coin      币种名字
 * @apiBody {Number} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
 * @apiBody {String} isForce      是否强制交易
 * @apiBody {String} [captcha]      如果是无需从设备签名的交易，则需要验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
            "from": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
            "to": "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb",
            "coin":"dw20",
            "amount": 123,
            "expireAt": 1708015513000
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {object} data                     订单结果.
* @apiSuccess {number} data.0                交易序列号.
* @apiSuccess {string} [data.1]                待签名数据(txid).
* @apiSampleRequest http://120.232.251.101:8066/wallet/preSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    to: String,
    coin: String,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
    is_forced: bool,
    captcha: Option<String>
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/preSendMoney")]
async fn pre_send_money(
    request: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    debug!("req_params:: {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::pre_send_money::req(request, request_data.0).await)
}


/**
 * @api {post} /wallet/preSendMoneyToSub 主钱包发起给子账户的预交易
 * @apiVersion 0.0.1
 * @apiName PreSendMoneyToSub
 * @apiGroup Wallet
 * @apiBody {String} to      收款方ID
 * @apiBody {String=BTC,ETH,USDT,USDC,DW20,CLY} coin      币种名字
 * @apiBody {Number} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
 * @apiBody {String} [captcha]      如果是无需从设备签名的交易，则需要验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
            "from": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
            "to": "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb",
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
* @apiSampleRequest http://120.232.251.101:8066/wallet/preSendMoneyToSub
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyToSubRequest {
    to: String,
    coin: String,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
    captcha: Option<String>
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/preSendMoneyToSub")]
async fn pre_send_money_to_sub(
    request: HttpRequest,
    request_data: web::Json<PreSendMoneyToSubRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::pre_send_money_to_sub::req(request, request_data.0).await)
}

/**
 * @api {post} /wallet/reactPreSendMoney 接受者收款确认
 * @apiVersion 0.0.1
 * @apiName reactPreSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} txIndex    交易序列号
 * @apiBody {bool} isAgreed    是否同意接收
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/reactPreSendMoney
   -d ' {
        "txIndex": 1,
        "isAgreed": true
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/reactPreSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReactPreSendMoney {
    tx_index: u32,
    is_agreed: bool,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/reactPreSendMoney")]
async fn react_pre_send_money(
    request: HttpRequest,
    request_data: web::Json<ReactPreSendMoney>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::react_pre_send_money::req(request, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/reconfirmSendMoney 发起方打款二次确认
 * @apiVersion 0.0.1
 * @apiName reconfirmSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} txIndex    交易序列号
 * @apiBody {String} confirmedSig    再确认就传签名结果,取消的话用cancelSendMoney的接口
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/reconfirmSendMoney
   -d ' {
        "deviceId":  "1",
        "txIndex": 1,
        "confirmedSig": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e6
                     83ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4e4c12e677ce
                35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600"
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/reconfirmSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconfirmSendMoneyRequest {
    tx_index: u32,
    confirmed_sig: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/reconfirmSendMoney")]
async fn reconfirm_send_money(
    request: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::reconfirm_send_money::req(request, request_data).await)
}





/**
 * @api {post} /wallet/cancelSendMoney   发起方取消交易
 * @apiVersion 0.0.1
 * @apiName CancelSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} txIndex    交易序列号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/cancelSendMoney
   -d ' {
        "deviceId":  "1",
        "txIndex": 1,
        "confirmedSig": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e6
                     83ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4e4c12e677ce
                35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600"
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/canceSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMoneyRequest {
    tx_index: u32,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/cancelSendMoney")]
async fn cancel_send_money(
    request: HttpRequest,
    request_data: web::Json<CancelSendMoneyRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::cancel_send_money::req(request, request_data).await)
}


/**
 * @api {post} /wallet/subSendToMain 子账户给主账户打款
 * @apiVersion 0.0.1
 * @apiName SubSendToMain
 * @apiGroup Wallet
 * @apiBody {String} sub_sig    子账户签名（pubkey+sig）
 * @apiBody {String} coin       交易币种
 * @apiBody {number} amount     交易数量
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/reconfirmSendMoney
   -d ' {
        "coin":  dw20,
        "amount": 1,
        "sub_sig": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e6
                     83ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4e4c12e677ce
                35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600"
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/reconfirmSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubSendToMainRequest {
    sub_sig: String,
    coin: String,
    amount: u128,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/subSendToMain")]
async fn sub_send_to_main(
    request: HttpRequest,
    request_data: web::Json<SubSendToMainRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::sub_send_to_main::req(request, request_data).await)
}

/**
 * @api {post} /wallet/uploadServantSig 上传从密钥的多签签名
 * @apiVersion 0.0.1
 * @apiName UsploadServantSig
 * @apiGroup Wallet
 * @apiBody {Number} txIndex    交易序列号
 * @apiBody {String} signature  pubkey和签名结果的拼接
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/uploadServantSig
   -d ' {
        "txIndex": 1,
        "signature": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe
3ac02e2bff9ee3e683ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4
e4c12e677ce35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/wallet/uploadServantSig
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadTxSignatureRequest {
    tx_index: u32,
    signature: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/uploadServantSig")]
async fn upload_servant_sig(
    request: HttpRequest,
    request_data: web::Json<UploadTxSignatureRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::upload_servant_sig::req(request, request_data).await)
}

/**
 * @api {post} /wallet/addServant 主设备添加从公钥
 * @apiVersion 0.0.1
 * @apiName addServant
 * @apiGroup Wallet
 * @apiBody {String} servantPubkey   从公钥
 * @apiBody {String} servantPrikeyEncrypedByPassword   经密码加密后的从私钥
 * @apiBody {String} servantPrikeyEncrypedByAnswer   经问答加密后的从私钥
 * @apiBody {String} holderDeviceId   指定持有从私钥的设备id
 * @apiBody {String} holderDeviceBrand   指定持有从私钥的设备型号

 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "servantPubkey": "123",
             "servantPrikeyEncrypedByPassword": "12345",
             "servantPrikeyEncrypedByAnswer": "12345",
             "holderDeviceId": "123",
             "holderDeviceBrand": "Apple",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/addServant
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddServantRequest {
    servant_pubkey: String,
    servant_prikey_encryped_by_password: String,
    servant_prikey_encryped_by_answer: String,
    holder_device_id: String,
    holder_device_brand: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/addServant")]
async fn add_servant(
    req: HttpRequest,
    request_data: web::Json<AddServantRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::add_servant::req(req, request_data.0).await)
}

/**
 * @api {post} /wallet/newcommerSwitchServant 在主设备上选择新设备替换从设备
 * @apiVersion 0.0.1
 * @apiName NewcommerSwitchServant
 * @apiGroup Wallet
 * @apiBody {String} oldServantPubkey   要被替换的从公钥
 * @apiBody {String} newServantPubkey   新晋从公钥
 * @apiBody {String} newServantPrikeyEncrypedByPassword   新晋从公钥对应的密钥被密码加密
 * @apiBody {String} newServantPrikeyEncrypedByAnswer   新晋从公钥对应的密钥被问答加密
 * @apiBody {String} newDeviceId   新晋持有从公钥的设备ID

 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/newcommerSwitchServant
   -d ' {"“}'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/newcommerSwitchServant
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewcommerSwitchServantRequest {
    old_servant_pubkey: String,
    new_servant_pubkey: String,
    new_servant_prikey_encryped_by_password: String,
    new_servant_prikey_encryped_by_answer: String,
    new_device_id: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/newcommerSwitchServant")]
async fn newcommer_switch_servant(
    req: HttpRequest,
    request_data: web::Json<NewcommerSwitchServantRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::newcommer_switch_servant::req(req, request_data.0).await)
}

/**
 * @api {post} /wallet/removeServant 在主设备上删除从设备
 * @apiVersion 0.0.1
 * @apiName RemoveServant
 * @apiGroup Wallet
 * @apiBody {String} servantPubkey   待删除从公钥

 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/removeServant
   -d ' {"“}'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/removeServant
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveServantRequest {
    servant_pubkey: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/removeServant")]
async fn remove_servant(
    req: HttpRequest,
    request_data: web::Json<RemoveServantRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::remove_servant::req(req, request_data.0).await)
}

/***
* @api {post} /wallet/servantSavedSecret 从设备告知服务端密钥已保存
* @apiVersion 0.0.1
* @apiName ServantSavedSecret
* @apiGroup Wallet
* @apiBody {String} servantPubkey   从公钥
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/servantSavedSecret
  -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/servantSavedSecret
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/servantSavedSecret")]
async fn servant_saved_secret(
    request: HttpRequest,
) -> impl Responder {
    gen_extra_respond(handlers::servant_saved_secret::req(request).await)
}

/**
 * @api {post} /wallet/addSubaccount 添加子钱包
 * @apiVersion 0.0.1
 * @apiName addSubaccount
 * @apiGroup Wallet
 * @apiBody {String} subaccountPubkey                   从公钥
 * @apiBody {String} subaccountPrikeyEncrypedByPassword      密码加密后的从私钥
 * @apiBody {String} subaccountPrikeyEncrypedByAnswer   问答加密后的从私钥
 * @apiBody {number} holdValueLimit   持仓上限
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/addSubaccount
   -d ' {
             "deviceId": "1",
             "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/addServant
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSubaccountRequest {
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_password: String,
    subaccount_prikey_encryped_by_answer: String,
    hold_value_limit: u128,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/addSubaccount")]
async fn add_subaccount(
    req: HttpRequest,
    request_data: web::Json<AddSubaccountRequest>,
) -> impl Responder {
    debug!("req_params:: {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::add_subaccount::req(req, request_data.0).await)
}


/**
 * @api {post} /wallet/removeSubaccount 删除子钱包
 * @apiVersion 0.0.1
 * @apiName RemoveSubaccount
 * @apiGroup Wallet
 * @apiBody {String} accountId                   待删除的钱包id
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/addSubaccount
   -d ' {
             "deviceId": "1",
             "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/removeSubaccount
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSubaccountRequest {
    account_id: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/removeSubaccount")]
async fn remove_subaccount(
    req: HttpRequest,
    request_data: web::Json<RemoveSubaccountRequest>,
) -> impl Responder {
    debug!("req_params:: {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::remove_subaccount::req(req, request_data.0).await)
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigRankExternal {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

/**
 * @api {post} /wallet/updateStrategy 更新主钱包多签梯度
 * @apiVersion 0.0.1
 * @apiName updateStrategy
 * @apiGroup Wallet
 * @apiBody {Object[]} strategy   策略内容
 * @apiBody {Number} strategy.min   档位最小值(开区间)
 * @apiBody {Number} strategy.maxEq  档位最大值(闭区间)
 * @apiBody {Number} strategy.sigNum   所需签名数量
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/updateStrategy
   -d '  {
             "strategy": [{"min": 0, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200, "sigNum": 1}]
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/updateStrategy
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategy {
    strategy: Vec<MultiSigRankExternal>,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/updateStrategy")]
async fn update_strategy(
    req: HttpRequest,
    request_data: web::Json<UpdateStrategy>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::update_strategy::req(req, request_data).await)
}


/**
 * @api {post} /wallet/setFeesPriority 设置手续费币种扣减顺序
 * @apiVersion 0.0.1
 * @apiName SetFeesPriority
 * @apiGroup Wallet
 * @apiBody {String[]=USDT,BTC,ETH,DW20,USDC} feesPriority   币种优先级

 * @apiBody {Number} strategy.sigNum   所需签名数量
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/setFeesPriority
   -d '  {
             "strategy": [{"min": 0, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200, "sigNum": 1}]
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/setFeesPriority
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetFeesPriorityRequest {
    fees_priority: Vec<String>,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/setFeesPriority")]
async fn set_fees_priority(
    req: HttpRequest,
    request_data: web::Json<SetFeesPriorityRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::set_fees_priority::req(req, request_data).await)
}



/**
 * @api {post} /wallet/updateSubaccountHoldLimit 更新子钱包的持仓额度
 * @apiVersion 0.0.1
 * @apiName UpdateSubaccountHoldLimit
 * @apiGroup Wallet
 * @apiBody {String} subaccount   待修改的子钱包id
 * @apiBody {Number} limit         待更新的额度
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/updateSubaccountHoldLimit
   -d '  {
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/updateSubaccountHoldLimit
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubaccountHoldLimitRequest {
    subaccount: String,
    limit: u128,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/updateSubaccountHoldLimit")]
async fn update_subaccount_hold_limit(
    req: HttpRequest,
    request_data: web::Json<UpdateSubaccountHoldLimitRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::update_subaccount_hold_limit::req(req, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/updateSecurity 更新安全密码
 * @apiVersion 0.0.1
 * @apiName UpdateSecurity
 * @apiGroup Wallet
 * @apiBody {String} anwserIndexes    新的安全问题
 * @apiBody {String} captcha        重设后的安全数据
 * @apiBody {Object[]} secrets        重设后的安全数据
 * @apiBody {String} secrets.pubkey    更新的钱包公钥
 * @apiBody {String} secrets.encryptedPrikeyByPassword  重设后被密码加密的私钥
 * @apiBody {String} secrets.encryptedPrikeyByAnswer    重设后被问答加密的私钥
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/updateStrategy
   -d '  {
             "strategy": [{"min": 0, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200, "sigNum": 1}]
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/updateStrategy
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretStoreTmp1 {
    pub pubkey: String,
    pub encrypted_prikey_by_password: String,
    pub encrypted_prikey_by_answer: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecurityRequest {
    anwser_indexes: String,
    secrets: Vec<SecretStoreTmp1>,
    captcha:String
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/updateSecurity")]
async fn update_security(
    req: HttpRequest,
    request_data: web::Json<UpdateSecurityRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::update_security::req(req, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/createMainAccount 创建主钱包
 * @apiVersion 0.0.1
 * @apiName CreateMainAccount
 * @apiGroup Wallet
 * @apiBody {String} masterPubkey                 主钱包master公钥
 * @apiBody {String} masterPrikeyEncryptedByPassword   密码加密的master私钥
 * @apiBody {String} masterPrikeyEncryptedByAnswer   问答加密的master私钥
 * @apiBody {String} subaccountPubkey              子钱包
 * @apiBody {String} subaccountPrikeyEncrypedByPassword   密码加密的子钱包私钥
 * @apiBody {String} subaccountPrikeyEncrypedByAnswer  问答加密的子钱包私钥
 * @apiBody {String} anwserIndexes               安全问答的序列号
 * @apiBody {String} captcha                 验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/createMainAccount
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/createMainAccount
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMainAccountRequest {
    master_pubkey: String,
    master_prikey_encrypted_by_password: String,
    master_prikey_encrypted_by_answer: String,
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_password: String,
    subaccount_prikey_encryped_by_answer: String,
    anwser_indexes: String,
    captcha: String
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/createMainAccount")]
async fn create_main_account(
    req: HttpRequest,
    request_data: web::Json<CreateMainAccountRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::create_main_account::req(req, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/faucetClaim 领取测试币
 * @apiVersion 0.0.1
 * @apiName faucetClaim
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/putPendingPubkey
   -d '  {
             "encryptedPrikey": "a06d01c1c74f33b4558454dbb863e90995543521fd7fc525432fc58b705f8cef19ae808dec479e1516ffce8ab2a0af4cec430d56f86f70e48f1002b912709f89",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/faucetClaim
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/faucetClaim")]
async fn faucet_claim(req: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::faucet_claim::req(req).await)
}

/**
 * @api {get} /wallet/balanceList 返回支持的资产的余额信息
 * @apiVersion 0.0.1
 * @apiName balanceList
 * @apiGroup Wallet
 * @apiQuery {String=Main,AllSub,Single(1234)} kind 
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/balanceList?kind=Main
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {object[]} data                币种和余额列表.
* @apiSuccess {string} data.0                钱包id.
* @apiSuccess {object[]} data.1              钱包的各币种余额 .
* @apiSuccess {string} data.1.account_id                钱包id和data.0一致.
* @apiSuccess {string=BTC,ETH,USDT,USDC,CLY,DW20} data.1.coin             币种名称.
* @apiSuccess {string} data.1.total_balance                      总余额.
* @apiSuccess {string} data.1.available_balance                  可用余额.
* @apiSuccess {string} data.1.freezn_amount                      冻结数量.
* @apiSampleRequest http://120.232.251.101:8066/wallet/balanceList
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceListRequest {
    kind: AccountType,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/balanceList")]
async fn balance_list(req: HttpRequest,request_data: web::Query<BalanceListRequest>) -> impl Responder {
    gen_extra_respond(handlers::balance_list::req(req,request_data.0).await)
}



/**
 * @api {get} /wallet/txList 账单详情
 * @apiVersion 0.0.1
 * @apiName txList
 * @apiGroup Wallet
 * @apiQuery {String=Sender,Receiver} TransferRole  交易中的角色，对应ui的转出和收款栏
 * @apiQuery {String}                 [counterParty]   交易对手方
 * @apiQuery {number}                 perPage           每页的数量
 * @apiQuery {number}                 page            页数的序列号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/txList?
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {object[]} data                交易详情数组.
* @apiSuccess {Number} data.tx_index          交易索引号.
* @apiSuccess {object} data.transaction        交易详情.
* @apiSuccess {String} [data.tx_id]        链上交易id.
* @apiSuccess {String=dw20,cly} data.coin_type      币种名字
* @apiSuccess {String} data.from                发起方
* @apiSuccess {String} data.to                接收方
* @apiSuccess {Number} data.amount               交易量
* @apiSuccess {String} data.CoinTx.transaction.expireAt             交易截止时间戳
* @apiSuccess {String} [data.memo]                交易备注
* @apiSuccess {String=Created,SenderSigCompleted,ReceiverApproved,ReceiverRejected,SenderCanceled,SenderReconfirmed} data.CoinTx.transaction.status                交易状态
* @apiSuccess {String}  data.coin_tx_raw       币种转账的业务原始数据hex
* @apiSuccess {String} [data.chain_tx_raw]          链上交互的原始数据
* @apiSuccess {object[]} data.signatures       从设备签名详情
* @apiSuccess {String} data.signatures.pubkey         签名公钥
* @apiSuccess {String} data.signatures.sig            签名结果
* @apiSuccess {String} data.signatures.device_id      签名设备id
* @apiSuccess {String} data.signatures.device_brand   签名设备品牌
* @apiSuccess {String} data.updated_at         交易更新时间戳
* @apiSuccess {String} data.created_at         交易创建时间戳
* @apiSampleRequest http://120.232.251.101:8066/wallet/balanceList
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxListRequest {
    tx_role: String,
    counterparty: Option<String>,
    per_page: u32,
    page: u32,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/txList")]
async fn tx_list(req: HttpRequest,request_data: web::Query<TxListRequest>) -> impl Responder {
    gen_extra_respond(handlers::tx_list::req(req,request_data.0).await)
}



/**
 * @api {get} /wallet/getTx 单个账单详情
 * @apiVersion 0.0.1
 * @apiName GetTx
 * @apiGroup Wallet
 * @apiQuery {number}                 index            交易序列号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/getTx?index=1
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {object} data               交易详情.
* @apiSuccess {Number} data.tx_index          交易索引号.
* @apiSuccess {object} data.transaction        交易详情.
* @apiSuccess {String} [data.tx_id]        链上交易id.
* @apiSuccess {String=dw20,cly} data.coin_type      币种名字
* @apiSuccess {String} data.from                发起方
* @apiSuccess {String} data.to                接收方
* @apiSuccess {Number} data.amount               交易量
* @apiSuccess {String} data.expireAt             交易截止时间戳
* @apiSuccess {String} [data.memo]                交易备注
* @apiSuccess {String=Created,SenderSigCompleted,ReceiverApproved,ReceiverRejected,SenderCanceled,SenderReconfirmed} data.status                
    交易状态分别对应{转账订单创建、从设备签名准备完毕、接收者同意收款、接收者拒绝收款、发送方取消转账、发送方二次确认交易}
* @apiSuccess {String}  data.coin_tx_raw       币种转账的业务原始数据hex
* @apiSuccess {String} [data.chain_tx_raw]          链上交互的原始数据
* @apiSuccess {String}  data.need_sig_num         本次转账预估需要的签名数量
* @apiSuccess {object[]} data.signed_device       从设备签名详情
* @apiSuccess {String} data.signed_device.pubkey         签名公钥
* @apiSuccess {String} data.signed_device.device_id      签名设备id
* @apiSuccess {String} data.signed_device.device_brand   签名设备品牌
* @apiSuccess {object[]} data.unsigned_device       还没签名从设备
* @apiSuccess {String} data.unsigned_device.pubkey         签名公钥
* @apiSuccess {String} data.unsigned_device.device_id      签名设备id
* @apiSuccess {String} data.unsigned_device.device_brand   签名设备品牌
* @apiSuccess {String} data.updated_at         交易更新时间戳
* @apiSuccess {String} data.created_at         交易创建时间戳
* @apiSampleRequest http://120.232.251.101:8066/wallet/getTx
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTxRequest {
    index: u32,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getTx")]
async fn get_tx(req: HttpRequest,request_data: web::Query<GetTxRequest>) -> impl Responder {
    gen_extra_respond(handlers::get_tx::req(req,request_data.into_inner()).await)
}

/**
 * @api {get} /wallet/deviceList 返回设备信息列表
 * @apiVersion 0.0.1
 * @apiName deviceList
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/deviceList
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {object[]} data                 设备信息列表.
* @apiSuccess {string} data.id                设备id.
* @apiSuccess {string} data.user_id           设备当前所属用户id.
* @apiSuccess {string} data.state             （不关注）.
* @apiSuccess {string} data.brand             设备品牌.
* @apiSuccess {string} data.holder_confirm_saved   设备主钱包持有的master或者servant的pubkey.
* @apiSuccess {string=Master,Servant,Undefined} data.key_role           当前设备持有的key的类型

* @apiSampleRequest http://120.232.251.101:8066/wallet/deviceList
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/deviceList")]
async fn device_list(req: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::device_list::req(req).await)
}

/**
 * @api {post} /wallet/genNewcomerSwitchMaster 构建在新设备上和主设备身份互换的任务
 * @apiVersion 0.0.1
 * @apiName GenNewcomerSwitchMaster
 * @apiGroup Wallet
 * @apiBody {String} newcomerPubkey                 新晋主公钥
 * @apiBody {String}     captcha                 验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/genNewcomerSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSuccess {string} data.add_key_txid                增加主公钥对应的tx_id
* @apiSuccess {string} data.add_key_raw                 增加主公钥对应的tx_raw.
* @apiSuccess {string} data.delete_key_txid             删除主公钥对应的tx_id.
* @apiSuccess {string} data.delete_key_raw              删除主公钥对应的tx_raw.
* @apiSampleRequest http://120.232.251.101:8066/wallet/genNewcomerSwitchMaster
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenNewcomerSwitchMasterRequest {
    newcomer_pubkey: String,
    captcha:String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/genNewcomerSwitchMaster")]
async fn gen_newcomer_switch_master(
    req: HttpRequest,
    request_data: web::Json<GenNewcomerSwitchMasterRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        handlers::gen_newcomer_switch_master::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet/genServantSwitchMaster 构建在从设备上和主设备身份互换的任务
 * @apiVersion 0.0.1
 * @apiName GenServantSwitchMaster
 * @apiGroup Wallet
 * @apiBody {String}     captcha                 验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/genServantSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSuccess {string} data.add_key_txid                增加从公钥成为主公钥对应的tx_id
* @apiSuccess {string} data.add_key_raw                 增加从公钥成为主公钥对应的tx_raw.
* @apiSuccess {string} data.delete_key_txid             删除旧主公钥对应的tx_id.
* @apiSuccess {string} data.delete_key_raw              删除旧主公钥对应的tx_raw.
* @apiSampleRequest http://120.232.251.101:8066/wallet/genServantSwitchMaster
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenServantSwitchMasterRequest {
    captcha: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/genServantSwitchMaster")]
async fn gen_servant_switch_master(req: HttpRequest,
    request_data: web::Json<GenServantSwitchMasterRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::gen_servant_switch_master::req(req,request_data.into_inner()).await)
}



/**
 * @api {post} /wallet/getNeedSigNum   获取转账需要几个从设备签名
 * @apiVersion 0.0.1
 * @apiName getNeedSigNum
 * @apiGroup Wallet
 * @apiBody {String}     coin                  转账币种
 * @apiBody {String}     amount                转账数量
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/genServantSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSuccess {string} data.add_key_txid                增加从公钥成为主公钥对应的tx_id
* @apiSuccess {string} data.add_key_raw                 增加从公钥成为主公钥对应的tx_raw.
* @apiSuccess {string} data.delete_key_txid             删除旧主公钥对应的tx_id.
* @apiSuccess {string} data.delete_key_raw              删除旧主公钥对应的tx_raw.
* @apiSampleRequest http://120.232.251.101:8066/wallet/genServantSwitchMaster
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetNeedSigNumRequest {
    coin: String,
    amount: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/getNeedSigNum")]
async fn get_need_sig_num(req: HttpRequest,
    request_data: web::Json<GetNeedSigNumRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::get_need_sig_num::req(req,request_data.into_inner()).await)
}



/**
 * @api {post} /wallet/genSendMoney 构建send_money的交易数据
 * @apiVersion 0.0.1
 * @apiName GenSendMoney
 * @apiGroup Wallet
 * @apiBody {number}     tx_index                交易订单号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/GenSendMoney
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                 待签名的交易id.
* @apiSampleRequest http://120.232.251.101:8066/wallet/genSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenSendMoneyRequest {
    tx_index: u32,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/genSendMoney")]
async fn gen_send_money(req: HttpRequest,
    request_data: web::Json<GenSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::gen_send_money::req(req,request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/commitNewcomerSwitchMaster 提交在新设备上和主设备身份互换的任务
 * @apiVersion 0.0.1
 * @apiName commitNewcomerSwitchMaster
 * @apiGroup Wallet
 * @apiBody {String} newcomerPubkey                            新晋主公钥
 * @apiBody {String} addKeyRaw                                  增加主公钥对应的tx_raw
 * @apiBody {String} deleteKeyRaw                               删除主公钥对应的tx_raw
 * @apiBody {String} addKeySig                                   旧的主私钥签名增加主公钥的结果
 * @apiBody {String} deleteKeySig                                新私钥签名删除主公钥的结果
 * @apiBody {String} newcomerPrikeyEncryptedByPassword                 新晋主公钥对应的密钥的被安全密码加密的结果
 * @apiBody {String} newcomerPrikeyEncryptedByAnswer             新晋主公钥对应的密钥的被安全问答加密的结果
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/commitNewcomerSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/commitNewcomerSwitchMaster
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitNewcomerSwitchMasterRequest {
    newcomer_pubkey: String,
    add_key_raw: String,
    delete_key_raw: String,
    add_key_sig: String,
    delete_key_sig: String,
    newcomer_prikey_encrypted_by_password: String,
    newcomer_prikey_encrypted_by_answer: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/commitNewcomerSwitchMaster")]
async fn commit_newcomer_switch_master(
    req: HttpRequest,
    request_data: web::Json<CommitNewcomerSwitchMasterRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        handlers::commit_newcomer_replace_master::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet/commitServantSwitchMaster 提交在从设备上和主设备身份互换的任务
 * @apiVersion 0.0.1
 * @apiName commitServantSwitchMaster
 * @apiGroup Wallet
 * @apiBody {String} addKeyRaw                                  增加主公钥对应的tx_raw
 * @apiBody {String} deleteKeyRaw                               删除主公钥对应的tx_raw
 * @apiBody {String} addKeySig                                   旧主私钥签名增加主公钥对应的结果
 * @apiBody {String} deleteKeySig                                旧从私钥签名删除主公钥对应的结果
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/commitServantSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1,3007} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,HaveUncompleteTx} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/commitServantSwitchMaster
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitServantSwitchMasterRequest {
    add_key_raw: String,
    delete_key_raw: String,
    add_key_sig: String,
    delete_key_sig: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/commitServantSwitchMaster")]
async fn commit_servant_switch_master(
    req: HttpRequest,
    request_data: web::Json<CommitServantSwitchMasterRequest>,
) -> impl Responder {
    debug!("req_params::  {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        handlers::commit_servant_switch_master::req(req, request_data.into_inner()).await,
    )
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_message)
        .service(get_strategy)
        .service(get_fees_priority)
        .service(set_fees_priority)
        .service(pre_send_money)
        .service(react_pre_send_money)
        .service(reconfirm_send_money)
        .service(upload_servant_sig)
        .service(add_servant)
        .service(remove_servant)
        .service(add_subaccount)
        .service(remove_subaccount)
        .service(update_strategy)
        .service(create_main_account)
        .service(servant_saved_secret)
        .service(device_list)
        .service(balance_list)
        .service(gen_newcomer_switch_master)
        .service(commit_newcomer_switch_master)
        .service(gen_servant_switch_master)
        .service(commit_servant_switch_master)
        .service(newcommer_switch_servant)
        .service(get_secret)
        .service(update_security)
        .service(sub_send_to_main)
        .service(tx_list)
        .service(get_tx)
        .service(pre_send_money_to_sub)
        .service(update_subaccount_hold_limit)
        .service(cancel_send_money)
        .service(gen_send_money)
        .service(get_need_sig_num)
        .service(faucet_claim);
    //.service(remove_subaccount);
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
    use handlers::get_strategy::StrategyDataTmp;
    use models::account_manager::UserInfoView;
    use tracing::{debug, error, info};
    use crate::wallet::handlers::get_tx::CoinTxViewTmp2;
    use std::collections::HashMap;
    use crate::wallet::handlers::balance_list::AccountBalance;


    /*** 

#[actix_web::test]
async fn test_wallet_yunlong_fake_tx() {
    let app = init().await;
    let service = test::init_service(app).await;
    let (mut sender_master,
        mut sender_servant,
        mut sender_newcommer,
        mut receiver) 
    = gen_some_accounts_with_new_key();

    sender_master.user.token = Some("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjo4LCJkZXZpY2VfaWQiOiIyNGIyNDEyZDdlYmE1YTAwIiwiZGV2aWNlX2JyYW5kIjoiSFVBV0VJIFAzMCBQcm8iLCJpYXQiOjE3MTE5NjgzNjU2NjQsImV4cCI6NDg2NTU2ODM2NTY2NH0.D_ZN8nygFDp6mHH4V9fHb3uP8fZ3LHNhRKaPinhaon8".to_string());
    sender_master.wallet.main_account="2d0236ddd991efb6a518f4428eed10d61a5d59f59e7b222a9918a36624c94e47".to_string();

    
    sender_master.user.contact = "+86 16666666661".to_string();
    test_faucet_claim!(service, sender_master);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
  
    loop{
        let receiver = "fe82ad43cb6cb59b7e5a18bd8b38abf577316f03fb471ac82ebec49802cbf3e0".to_string();
        test_get_captcha_with_token!(service,sender_master,"PreSendMoney");    
        let pre_send_res = test_pre_send_money!(service,sender_master,receiver,"DW20",12,true,Some("000000".to_string()));
        assert!(pre_send_res.is_some());
    }    

}
*/
#[actix_web::test]
async fn test_wallet_fees_prioritys_op() {
    //todo: cureent is single, add multi_sig testcase
    let _app = init().await;
    let app = init().await;
    let service = test::init_service(app).await;
    let (mut sender_master,_,_,_) = gen_some_accounts_with_new_key();

    test_register!(service, sender_master);
    test_create_main_account!(service, sender_master);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    let priority1 = test_get_fees_priority!(service,sender_master).unwrap();
    println!("priority1___{:?}",priority1);
    test_set_fees_priority!(service,sender_master);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    let priority2 = test_get_fees_priority!(service,sender_master).unwrap();
    println!("priority2___{:?}",priority2);
}


#[actix_web::test]
async fn test_wallet_add_remove_subaccount() {
    //todo: cureent is single, add multi_sig testcase
    let _app = init().await;
    let app = init().await;
    let service = test::init_service(app).await;
    let (mut sender_master,_,_,_) = gen_some_accounts_with_new_key();

    test_register!(service, sender_master);
    test_create_main_account!(service, sender_master);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    let subacc = sender_master.wallet.subaccount.first().unwrap();
    let (new_sub_prikey,new_sub_pubkey) = ed25519_key_gen();
    test_add_subaccount!(service,sender_master,new_sub_pubkey);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    let strategy = test_get_strategy!(service,sender_master).unwrap();
    println!("___{:?}",strategy);
    assert_eq!(strategy.subaccounts.get(&new_sub_pubkey).unwrap().hold_value_limit,10000);

    test_remove_subaccount!(service,sender_master,new_sub_pubkey);
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    let strategy = test_get_strategy!(service,sender_master).unwrap();
    assert!(strategy.subaccounts.get(&new_sub_pubkey).is_none());
}

    #[actix_web::test]
    async fn test_wallet_update_subaccount_hold_limit_ok() {
        //todo: cureent is single, add multi_sig testcase
        let _app = init().await;
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,_,_,_) = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let subacc = sender_master.wallet.subaccount.first().unwrap();
        test_update_subaccount_hold_limit!(service,sender_master,subacc,12);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let strategy = test_get_strategy!(service,sender_master).unwrap();
        println!("___{:?}",strategy);
        assert_eq!(strategy.subaccounts.get(subacc).unwrap().hold_value_limit,12);
    }

    #[actix_web::test]
    async fn test_wallet_force_transfer_with_servant() {
        //todo: cureent is single, add multi_sig testcase
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,mut sender_servant,_,mut receiver) = gen_some_accounts_with_new_key();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        test_register!(service, sender_master);
        test_register!(service, receiver);
        test_login!(service, sender_servant);
        test_create_main_account!(service, sender_master);
        test_create_main_account!(service, receiver);
        test_faucet_claim!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service, sender_master, sender_servant);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::NewcomerBecameSevant(_secret) = res.first().unwrap() {
            test_servant_saved_secret!(service,sender_servant);
        }   

        let pre_send_res = test_pre_send_money!(service,sender_master,receiver.wallet.main_account,"DW20",12,true,None::<String>);
        assert!(pre_send_res.is_some());
        
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(tx.tx_type, TxType::Forced);
            //local sign
            let signature = common::encrypt::ed25519_gen_pubkey_sign(
                &sender_servant.wallet.prikey.unwrap(),
                &tx.coin_tx_raw,
            ).unwrap();
           test_upload_servant_sig!(service,sender_servant,index,signature);
        }

        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::ReceiverApproved);
            assert_eq!(tx.tx_type, TxType::Forced);
            //local sign
            let signature = common::encrypt::ed25519_sign_hex(
                sender_master.wallet.prikey.as_ref().unwrap(),
                tx.tx_id.as_ref().unwrap(),
            ).unwrap();
            test_reconfirm_send_money!(service,sender_master,index,signature);
        }
    }
    
    #[actix_web::test]
    async fn test_wallet_force_transfer_without_servant() {
        //todo: cureent is single, add multi_sig testcase
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,_,_,mut receiver) = gen_some_accounts_with_new_key();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        //let receive = a336dc50a8cef019d92c3c80c92a2a9d3842c95576d544286d166f1501a2351b
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        test_register!(service, receiver);
        test_create_main_account!(service, receiver);
        test_faucet_claim!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        test_get_captcha_with_token!(service,sender_master,"PreSendMoney");
        let (index,txid) = test_pre_send_money!(
            service,sender_master,
            receiver.wallet.main_account,"DW20",1,true,Some("000000".to_string())
        ).unwrap();

        println!("txid {:?}",txid);
        let signature = common::encrypt::ed25519_sign_hex(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &txid.unwrap(),
        ).unwrap();
        test_reconfirm_send_money!(service,sender_master,index,signature);
    }

    #[actix_web::test]
    async fn test_wallet_replace_servant() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,mut sender_servant,mut sender_newcommer,_receiver) = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_login!(service, sender_servant);
        test_login!(service, sender_newcommer);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service, sender_master, sender_servant);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_newcommer_switch_servant!(service, sender_master, sender_servant,sender_newcommer);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
    }

    #[actix_web::test]
    async fn test_wallet_servant_switch_master() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,mut sender_servant,_sender_newcommer,_receiver) = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_login!(service, sender_servant);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service, sender_master, sender_servant);
        test_faucet_claim!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_get_captcha_with_token!(service,sender_servant,"ServantSwitchMaster");
        let gen_res = test_gen_servant_switch_master!(service,sender_servant);
        let add_key_sig = common::encrypt::ed25519_sign_hex(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &gen_res.as_ref().unwrap().add_key_txid,
        ).unwrap();

        let delete_key_sig = common::encrypt::ed25519_sign_hex(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &gen_res.as_ref().unwrap().delete_key_txid,
        ).unwrap();
        test_commit_servant_switch_master!(service,sender_servant,gen_res,add_key_sig,delete_key_sig);
        let device_lists: Vec<DeviceInfo> = test_get_device_list!(service,sender_servant).unwrap();
        println!("{},,,{:?}", line!(), device_lists);
    }

    #[actix_web::test]
    async fn test_wallet_main_send_money_to_sub() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,_sender_servant,_sender_newcommer,_receiver) = gen_some_accounts_with_new_key();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        test_faucet_claim!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        test_get_captcha_with_token!(service,sender_master,"PreSendMoneyToSub");
        //step3: master: pre_send_money
        test_pre_send_money_to_sub!(
            service,
            sender_master,
            sender_master.wallet.subaccount.first().unwrap(),
            "DW20",
            12
        );

        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::SenderSigCompletedAndReceiverIsSub);
            //local sign
            let signature = common::encrypt::ed25519_gen_pubkey_sign(
                sender_master.wallet.prikey.as_ref().unwrap(),
                //区别于普通转账，给子账户的签coin_tx_raw
                &tx.coin_tx_raw,
            ).unwrap();
            test_reconfirm_send_money!(service,sender_master,index,signature);
        }
    }
    use std::str::FromStr;
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct SubAccCoinTx {
        pub coin_id: String,
        pub amount: u128,
    }
    
    #[actix_web::test]
    async fn test_wallet_sub_send_money_to_main() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,_sender_servant,_sender_newcommer,_receiver) = gen_some_accounts_with_new_key();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
       tokio::time::sleep(std::time::Duration::from_millis(3000)).await;


            //sub to main
            let coin_tx = SubAccCoinTx{
                amount: 11,
                //todo: 
                coin_id: "dw20".to_string(),
            };
            let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();

            //todo: 也许子账户转主账户也需要落表
            let coin_tx_hex_str = hex::encode(coin_tx_str.as_bytes());    
            let signature = common::encrypt::ed25519_sign_hex(
                sender_master.wallet.sub_prikey.clone().unwrap().first().unwrap(),
                &coin_tx_hex_str,
            ).unwrap();

            let signature = format!(
                "{}{}",
                sender_master.wallet.subaccount.first().unwrap(),
                signature
            );
            test_sub_send_to_master!(service,sender_master,signature,"DW20",11);
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct SecretStoreTmp2 {
        pub pubkey: String,
        pub encryptedPrikeyByPassword: String,
        pub encryptedPrikeyByAnswer: String,
    }
    #[actix_web::test]
    async fn test_wallet_change_security() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,mut sender_servant,_sender_newcommer,_receiver) = gen_some_accounts_with_new_key();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_login!(service, sender_servant);
        test_add_servant!(service, sender_master, sender_servant);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let secrets = test_get_secret!(service, sender_master, "all").unwrap();
        println!("res {:?}", secrets);

        //re-encrypt prikey
        let secrets:Vec<SecretStoreTmp2> = secrets.iter().map(|s| {
            SecretStoreTmp2{
                pubkey: s.pubkey.clone(),
                encryptedPrikeyByPassword:"updated_encrypted".to_string(),
                encryptedPrikeyByAnswer:"updated_encrypted".to_string()
            }
        }).collect();
        //claim
        test_update_security!(service, sender_master, secrets);
    }

    #[actix_web::test]
    async fn test_wallet_newcommer_replace_master() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,
            _sender_servant,
            mut sender_newcommer,
            _receiver) 
        = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        //todo：当前例子中不注册也能跑通，要加限制条件，必须已经注册
        test_login!(service, sender_newcommer);
        test_create_main_account!(service, sender_master);
        test_faucet_claim!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;


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
    }

    #[actix_web::test]
    async fn test_wallet_get_all() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,
            _sender_servant,
            _sender_newcommer,
            _receiver) 
        = gen_some_accounts_with_new_key();
    
        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let balances = test_get_balance_list!(service, sender_master,"Main").unwrap();
        println!("list {:?}", balances);

        let secrets = test_get_secret!(service, sender_master, "all").unwrap();
        println!("secrets {:?}", secrets);

        let txs = test_tx_list!(service, sender_master,"Sender",None::<String>,100,1).unwrap();
        println!("txs__ {:?}", txs);
    }

    #[actix_web::test]
    async fn test_wallet_faucet_ok() {
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master,
            _sender_servant,
            _sender_newcommer,
            _receiver) 
        = gen_some_accounts_with_new_key();

        test_register!(service, sender_master);

        let balances1 = test_get_balance_list!(service, sender_master,"Main").unwrap();
        println!("list {:?}", balances1);

        test_create_main_account!(service, sender_master);

        //claim
        test_faucet_claim!(service, sender_master);

        //balance
        let balances2 = test_get_balance_list!(service, sender_master,"Main").unwrap();
        println!("list {:?}", balances2);
    }

    /*** 
    #[actix_web::test]
    async fn test_wallet_all_braced_wallet_ok_with_fix_key() {
        let sender_master = simulate_sender_master();
        let receiver = simulate_receiver();
        let sender_servant = simulate_sender_servant();


        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        test_all_braced_wallet_ok(sender_master, receiver, sender_servant).await;
    }
    */

    #[actix_web::test]
    async fn test_wallet_all_braced_wallet_ok_with_new_key() {
        //fixme: currently used service mode is from environment ,not init's value
         let (sender_master,
            sender_servant,
            _sender_newcommer,
            receiver) 
         = gen_some_accounts_with_new_key();

        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20).unwrap();
        coin_cli.send_coin(&sender_master.wallet.main_account, 13u128).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        test_all_braced_wallet_ok(sender_master, receiver, sender_servant).await;
    }

    //#[actix_web::test]
    async fn test_all_braced_wallet_ok(
        mut sender_master: TestWulianApp2,
        mut receiver: TestWulianApp2,
        mut sender_servant: TestWulianApp2,
    ) {
        let app = init().await;
        let service = test::init_service(app).await;
        //init: get token by register or login
        test_register!(service, sender_master);
        test_register!(service, receiver);
        test_login!(service, sender_servant);
        test_create_main_account!(service, receiver);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let receiver_strategy = test_get_strategy!(service, receiver).unwrap();
        println!("receiver strategy {:?}",receiver_strategy);

        test_create_main_account!(service, sender_master);
        test_faucet_claim!(service, sender_master);


        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service, sender_master, sender_servant);

        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        //给链上确认一些时间
        //step2.1: 
        test_update_strategy!(service,sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        //step2.2: check sender's new strategy
        let sender_strategy = test_get_strategy!(service, sender_master).unwrap();
        println!("{},,,{:?}", line!(), sender_strategy);

        //step2.3: get message of becoming servant,and save encrypted prikey
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::NewcomerBecameSevant(secret) = res.first().unwrap() {
            sender_servant.wallet.prikey = Some(secret.encrypted_prikey_by_password.clone());
            test_servant_saved_secret!(service,sender_servant);
        }

        //step2.4: get device list
        let device_lists: Vec<DeviceInfo> = test_get_device_list!(service,sender_servant).unwrap();
        println!("{},,,{:?}", line!(), device_lists);

        //step3: master: pre_send_money
        let res = test_pre_send_money2!(service,sender_master,receiver.user.contact,"DW20",12,false);
        assert!(res.is_some());
        let tx = test_get_tx!(service,sender_master,res.unwrap().0);
        println!("txxxx__{:?}",tx.unwrap());

        //step3.1: 对于created状态的交易来说，主设备不处理，从设备上传签名
        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(_index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "this device hold  master key,and do nothing for 'Created' tx"
            );
        }
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                sender_strategy.servant_pubkeys.first().unwrap(),
                "this device hold  servant key,need to sig for 'Created' tx"
            );

            //step4: upload sender servant sign
            //local sign
            let signature = common::encrypt::ed25519_sign_hex(
                &sender_servant.wallet.prikey.unwrap(),
                &tx.coin_tx_raw,
            ).unwrap();
            let signature = format!(
                "{}{}",
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                signature
            );

            //upload_servant_sig
           test_upload_servant_sig!(service,sender_servant,index,signature);
        }

        //step5: receiver get notice and react it
        let res = test_search_message!(service, receiver).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::SenderSigCompleted);
            assert_eq!(
                receiver.wallet.pubkey.unwrap(),
                receiver_strategy.master_pubkey,
                "only master_key can ratify or refuse it"
            );
            test_react_pre_send_money!(service,receiver,index,true);
        }

        let txs = test_tx_list!(service, sender_master,"sender",None::<String>,100,1).unwrap();
        println!("txs_0003__ {:?}", txs);


        //step6: sender_master get notice and react it
        //todo: 为了减少一个接口以及减掉客户端交易组装的逻辑，在to账户确认的时候就生成了txid和raw_data,所以master只有1分钟的确认时间
        //超过了就链上过期（非多签业务过期）
        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::ReceiverApproved);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "only sender_master_key can reconfirm or refuse it"
            );

            //local sign
            let signature = common::encrypt::ed25519_sign_hex(
                sender_master.wallet.prikey.as_ref().unwrap(),
                tx.tx_id.as_ref().unwrap(),
            ).unwrap();
            test_reconfirm_send_money!(service,sender_master,index,signature);
        }

        let txs_success = get_tx_status_on_chain(vec![1u64, 2u64]).await;
        println!("txs_success {:?}", txs_success);
    }



    async fn test_wallet_all_braced_wallet_ok_force(
        mut sender_master: TestWulianApp2,
        mut receiver: TestWulianApp2,
        mut sender_servant: TestWulianApp2,
    ) {
        let app = init().await;
        let service = test::init_service(app).await;
        //init: get token by register or login
        test_register!(service, sender_master);
        test_register!(service, receiver);
        test_login!(service, sender_servant);
        test_create_main_account!(service, receiver);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let receiver_strategy = test_get_strategy!(service, receiver).unwrap();
        println!("receiver strategy {:?}",receiver_strategy);

        test_create_main_account!(service, sender_master);


        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service, sender_master, sender_servant);

        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        //给链上确认一些时间
        //step2.1: 
        test_update_strategy!(service,sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        //step2.2: check sender's new strategy
        let sender_strategy = test_get_strategy!(service, sender_master).unwrap();
        println!("{},,,{:?}", line!(), sender_strategy);

        //step2.3: get message of becoming servant,and save encrypted prikey
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::NewcomerBecameSevant(secret) = res.first().unwrap() {
            sender_servant.wallet.prikey = Some(secret.encrypted_prikey_by_password.clone());
            test_servant_saved_secret!(service,sender_servant);
        }

        //step2.4: get device list
        let device_lists: Vec<DeviceInfo> = test_get_device_list!(service,sender_servant).unwrap();
        println!("{},,,{:?}", line!(), device_lists);

        //step3: master: pre_send_money
        test_get_captcha_with_token!(service,sender_master,"PreSendMoney");
        let res = test_pre_send_money!(service,sender_master,receiver.wallet.main_account,"DW20",12,true,None::<String>);
        assert!(res.is_some());
        //step3.1: 对于created状态的交易来说，主设备不处理，从设备上传签名
        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(_index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "this device hold  master key,and do nothing for 'Created' tx"
            );
        }
        let res = test_search_message!(service, sender_servant).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                sender_strategy.servant_pubkeys.first().unwrap(),
                "this device hold  servant key,need to sig for 'Created' tx"
            );

            //step4: upload sender servant sign
            //local sign
            let signature = common::encrypt::ed25519_sign_hex(
                &sender_servant.wallet.prikey.unwrap(),
                &tx.coin_tx_raw,
            ).unwrap();
            let signature = format!(
                "{}{}",
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                signature
            );

            //upload_servant_sig
           test_upload_servant_sig!(service,sender_servant,index,signature);
        }

        //step5: receiver get notice and react it
        let res = test_search_message!(service, receiver).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::SenderSigCompleted);
            assert_eq!(
                receiver.wallet.pubkey.unwrap(),
                receiver_strategy.master_pubkey,
                "only master_key can ratify or refuse it"
            );
            test_react_pre_send_money!(service,receiver,index,true);
        }

        let txs = test_tx_list!(service, sender_master,"sender",None::<String>,100,1).unwrap();
        println!("txs_0003__ {:?}", txs);

        //step6: sender_master get notice and react it
        //todo: 为了减少一个接口以及减掉客户端交易组装的逻辑，在to账户确认的时候就生成了txid和raw_data,所以master只有1分钟的确认时间
        //超过了就链上过期（非多签业务过期）
        let res = test_search_message!(service, sender_master).unwrap();
        if let AccountMessage::CoinTx(index, tx) = res.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::ReceiverApproved);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "only sender_master_key can reconfirm or refuse it"
            );

            //local sign
            let signature = common::encrypt::ed25519_sign_hex(
                sender_master.wallet.prikey.as_ref().unwrap(),
                tx.tx_id.as_ref().unwrap(),
            ).unwrap();
            test_reconfirm_send_money!(service,sender_master,index,signature);
        }

        let txs_success = get_tx_status_on_chain(vec![1u64, 2u64]).await;
        println!("txs_success {:?}", txs_success);
    }
}



