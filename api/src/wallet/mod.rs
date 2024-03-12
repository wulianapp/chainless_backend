//! account manager http service

mod handlers;
mod transaction;

use actix_web::{get, post, web, HttpRequest, Responder};

use serde::{Deserialize, Serialize};

use crate::utils::respond::gen_extra_respond;
//use crate::transaction::{get_all_message, get_user_message, insert_new_message, MessageType, update_message_status};

use crate::account_manager::{
    contact_is_used, get_captcha, login, register_by_email, register_by_phone, reset_password
};
use tracing::{debug, span, Level};

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
 * @apiSuccess {String} data.CoinTx.transaction.coin_tx_raw           币种转账的业务原始数据hex
 * @apiSuccess {String} [data.CoinTx.transaction.chain_tx_raw]          链上交互的原始数据
 * @apiSuccess {String[]} data.CoinTx.transaction.signatures         从设备对业务数据的签名
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
 * @apiQuery {String} accountId 钱包ID
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/searchMessageByAccountId?accounId
=2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
 * @apiSuccess {string=0,1} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {Object} data                          策略详情.
 * @apiSuccess {String} data.master_pubkey        主账户的maser的公钥
 * @apiSuccess {String[]} data.servant_pubkeys    主账户的servant的公钥组
 * @apiSuccess {String[]} data.subaccounts        子账户的公钥组
 * @apiSuccess {Object[]} [data.multi_sig_ranks]        转账额度对应签名数的档位.
 * @apiSuccess {Number} data.multi_sig_ranks.min       最小金额.
 * @apiSuccess {Number} data.multi_sig_ranks.max_eq        最大金额.
 * @apiSuccess {Number} data.multi_sig_ranks.sig_num        金额区间需要的最小签名数.
 * @apiSampleRequest http://120.232.251.101:8066/wallet/getStrategy
 */

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetStrategyRequest {
    account_id: String,
}
//todo: 删掉account_id，从数据库去拿
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getStrategy")]
async fn get_strategy(
    request: HttpRequest,
    request_data: web::Query<GetStrategyRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_strategy::req(request, request_data.into_inner()).await)
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetSecretRequest {
    pubkey: String
}
//todo: 删掉account_id，从数据库去拿
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/getSecret")]
async fn get_secret(
    request: HttpRequest,
    request_data: web::Query<GetSecretRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_secret::req(request, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/preSendMoney 主公钥发起预交易
 * @apiVersion 0.0.1
 * @apiName preSendMoney
 * @apiGroup Wallet
 * @apiBody {String} from    发起方多签钱包ID
 * @apiBody {String} to      收款方ID
 * @apiBody {String=BTC,ETH,USDT,USDC,DW20,CLY} coin      币种名字
 * @apiBody {Number} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
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
* @apiSampleRequest http://120.232.251.101:8066/wallet/preSendMoney
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    from: String,
    to: String,
    coin: String,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/preSendMoney")]
async fn pre_send_money(
    request: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::pre_send_money::req(request, request_data.0).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DirectSendMoneyRequest {
    tx_raw: String,
    signature: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/directSendMoney")]
async fn direct_send_money(
    request: HttpRequest,
    request_data: web::Json<DirectSendMoneyRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::direct_send_money::req(request, request_data).await)
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
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::react_pre_send_money::req(request, request_data.into_inner()).await)
}

/**
 * @api {post} /wallet/reconfirmSendMoney 发起方打款二次确认
 * @apiVersion 0.0.1
 * @apiName reconfirmSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} txIndex    交易序列号
 * @apiBody {String} [confirmedSig]    再确认就传签名结果，取消就不传这个字段
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
    confirmed_sig: Option<String>,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/reconfirmSendMoney")]
async fn reconfirm_send_money(
    request: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::reconfirm_send_money::req(request, request_data).await)
}

/**
 * @api {post} /wallet/uploadServantSig 上传从密钥的多签签名
 * @apiVersion 0.0.1
 * @apiName uploadServantSig
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
pub struct uploadTxSignatureRequest {
    tx_index: u32,
    signature: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/uploadServantSig")]
async fn upload_servant_sig(
    request: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::upload_servant_sig::req(request, request_data).await)
}

/**
 * @api {post} /wallet/addServant 主设备添加从公钥匙
 * @apiVersion 0.0.1
 * @apiName addServant
 * @apiGroup Wallet
 * @apiBody {String} mainAccount    主账户Id
 * @apiBody {String} servantPubkey   从公钥
 * @apiBody {String} servantPrikeyEncrypedByPwd   经密码加密后的从私钥
 * @apiBody {String} servantPrikeyEncrypedByAnswer   经问答加密后的从私钥
 * @apiBody {String} holderDeviceId   指定持有从私钥的设备id
 * @apiBody {String} holderDeviceBrand   指定持有从私钥的设备型号

 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "mainAccount": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "servantPubkey": "123",
             "servantPrikeyEncrypedByPwd": "12345",
             "servantPrikeyEncrypedByAnswer": "12345",
             "holderDeviceId": "123",
             "holderDeviceBrand": "Apple",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/addServant
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddServantRequest {
    main_account: String,
    servant_pubkey: String,
    servant_prikey_encryped_by_pwd: String,
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
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::add_servant::req(req, request_data.0).await)
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceServantRequest {
    main_account: String,
    old_servant_pubkey: String,
    new_servant_pubkey: String,
    new_servant_prikey_encryped_by_pwd: String,
    new_servant_prikey_encryped_by_answer: String,
    new_device_id: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/replaceServant")]
async fn replace_servant(
    req: HttpRequest,
    request_data: web::Json<ReplaceServantRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::replace_servant::req(req, request_data.0).await)
}



#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveServantRequest {
    servant_pubkey: String
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/removeServant")]
async fn remove_servant(
    req: HttpRequest,
    request_data: web::Json<RemoveServantRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::remove_servant::req(req, request_data.0).await)
}

/***
* @api {post} /wallet/servantSavedSecret 从设备通知服务端已经本地保存好
* @apiVersion 0.0.1
* @apiName servantSavedSecret
* @apiGroup Wallet
* @apiBody {String} servantPubkey   从公钥
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
  -d ' {
            "mainAccount": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
            "servantPubkey": "123",
            "servantPrikeyEncrypedByPwd": "12345",
            "servantPrikeyEncrypedByAnswer": "12345",
            "holderDeviceId": "123",
            "holderDeviceBrand": "Apple",
          }'
  -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/addServant
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServantSavedSecretRequest {
    servant_pubkey: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/servantSavedSecret")]
async fn servant_saved_secret(
    request: HttpRequest,
    request_data: web::Json<ServantSavedSecretRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::servant_saved_secret::req(request, request_data.0).await)
}

/**
 * @api {post} /wallet/addSubaccount 添加子账户
 * @apiVersion 0.0.1
 * @apiName addSubaccount
 * @apiGroup Wallet 
 * @apiBody {String} mainAccount                        主账户id
 * @apiBody {String} subaccountPubkey                   从公钥
 * @apiBody {String} subaccountPrikeyEncrypedByPwd      密码加密后的从私钥
 * @apiBody {String} subaccountPrikeyEncrypedByAnswer   问答加密后的从私钥
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/preSendMoney
   -d ' {
             "accountId": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "deviceId": "1",
             "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/addServant
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSubaccountRequest {
    main_account: String,
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_pwd: String,
    subaccount_prikey_encryped_by_answer: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/addSubaccount")]
async fn add_subaccount(
    req: HttpRequest,
    request_data: web::Json<AddSubaccountRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::add_subaccount::req(req, request_data.0).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigRankExternal {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

/**
 * @api {post} /wallet/updateStrategy 更新主账户多签梯度
 * @apiVersion 0.0.1
 * @apiName updateStrategy
 * @apiGroup Wallet
 * @apiBody {String} accountId    主钱包Id
 * @apiBody {Object[]} strategy   策略内容
 * @apiBody {Number} strategy.min   档位最小值(开区间)
 * @apiBody {Number} strategy.maxEq  档位最大值(闭区间)
 * @apiBody {Number} strategy.sigNum   所需签名数量


 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/updateStrategy
   -d '  {
             "accountId": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "strategy": [{"min": 0, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200, "sigNum": 1}]
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/wallet/updateStrategy
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategy {
    account_id: String,
    strategy: Vec<MultiSigRankExternal>,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/updateStrategy")]
async fn update_strategy(
    req: HttpRequest,
    request_data: web::Json<UpdateStrategy>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::update_strategy::req(req, request_data).await)
}

/**
 * @api {post} /wallet/createMainAccount 创建主钱包
 * @apiVersion 0.0.1
 * @apiName createMainAccount
 * @apiGroup Wallet
 * @apiBody {String} masterPubkey                 主账户master公钥  
 * @apiBody {String} masterPrikeyEncryptedByPwd   密码加密的master私钥
 * @apiBody {String} masterPrikeyEncryptedByAnswer   问答加密的master私钥
 * @apiBody {String} subaccountPubkey              子账户
 * @apiBody {String} subaccountPrikeyEncrypedByPwd   密码加密的子账户私钥
 * @apiBody {String} subaccountPrikeyEncrypedByAnswer  问答加密的子账户私钥
 * @apiBody {String} signPwdHash               密码和问答拼接后的hash结果
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
* @apiSampleRequest http://120.232.251.101:8066/wallet/newMaster
*/

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMainAccountRequest {
    master_pubkey: String,
    master_prikey_encrypted_by_pwd: String,
    master_prikey_encrypted_by_answer: String,
    subaccount_pubkey: String,
    subaccount_prikey_encryped_by_pwd: String,
    subaccount_prikey_encryped_by_answer: String,
    sign_pwd_hash: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/createMainAccount")]
async fn create_main_account(
    req: HttpRequest,
    request_data: web::Json<CreateMainAccountRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
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
async fn faucet_claim(
    req: HttpRequest
) -> impl Responder {
    gen_extra_respond(handlers::faucet_claim::req(req).await)
}

/**
 * @api {get} /wallet/balanceList 返回支持的五种资产的余额信息
 * @apiVersion 0.0.1
 * @apiName balanceList
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet/balanceList
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {string=0,1} status_code         status code.
* @apiSuccess {string=Successfully,InternalError} msg
* @apiSuccess {string[]} data                币种和余额列表.
* @apiSampleRequest http://120.232.251.101:8066/wallet/balanceList
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/balanceList")]
async fn balance_list(req: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::balance_list::req(req).await)
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
* @apiSuccess {string} data.holder_confirm_saved   设备主账户持有的master或者servant的pubkey.
* @apiSuccess {string=Master,Servant,Undefined} data.key_role           当前设备持有的key的类型

* @apiSampleRequest http://120.232.251.101:8066/wallet/deviceList
*/
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/wallet/deviceList")]
async fn device_list(req: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::device_list::req(req).await)
}



#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenTxNewcomerReplaceMasterRequest {
    newcomer_pubkey: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/genTxNewcomerReplaceMaster")]
async fn gen_tx_newcomer_replace_master(
    req: HttpRequest,
    request_data: web::Json<GenTxNewcomerReplaceMasterRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::gen_newcomer_replace_master::req(req, request_data.into_inner()).await)
}



#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitNewcomerReplaceMasterRequest {
    newcomer_pubkey: String,
    add_key_raw: String,
    delete_key_raw: String,
    add_key_sig: String,
    delete_key_sig: String,
    newcomer_prikey_encrypted_by_pwd: String,
    newcomer_prikey_encrypted_by_answer: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/wallet/commitTxNewcomerReplaceMaster")]
async fn commit_newcomer_replace_master(
    req: HttpRequest,
    request_data: web::Json<CommitNewcomerReplaceMasterRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::commit_newcomer_replace_master::req(req, request_data.into_inner()).await)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    //            .service(account_manager::get_captcha)
    cfg.service(search_message)
        .service(get_strategy)
        .service(get_secret)
        .service(pre_send_money)
        .service(direct_send_money)
        .service(react_pre_send_money)
        .service(reconfirm_send_money)
        .service(upload_servant_sig)
        .service(add_servant)
        .service(remove_servant)
        .service(add_subaccount)
        .service(update_strategy)
        .service(create_main_account)
        .service(servant_saved_secret)
        .service(device_list)
        .service(balance_list)
        .service(gen_tx_newcomer_replace_master)
        .service(commit_newcomer_replace_master)
        .service(replace_servant)
        .service(faucet_claim);
    //.service(remove_subaccount);
}

#[cfg(test)]
mod tests {
    use crate::utils::respond::BackendRespond;
    use crate::{
        test_add_servant, test_create_main_account, test_get_strategy, test_login, test_register, test_search_message, test_service_call
    };

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
    use blockchain::multi_sig::{ed25519_key_gen, StrategyData};
    use blockchain::multi_sig::{CoinTx, MultiSig};
    use common::data_structures::account_manager::UserInfo;
    use common::data_structures::secret_store::SecretStore;
    use common::data_structures::wallet::{AccountMessage, CoinTxStatus};
    use common::utils::math;
    use models::secret_store::SecretStoreView;
    // use log::{info, LevelFilter,debug,error};
    use handlers::get_strategy::StrategyDataTmp;
    use models::account_manager::UserInfoView;
    use tracing::{debug, error, info};
    use common::data_structures::wallet::CoinType;

    struct TestWallet {
        main_account: String,
        pubkey: Option<String>,
        prikey: Option<String>,
        subaccount: Vec<String>,
        sub_prikey: Option<Vec<String>>,
    }

    struct TestDevice {
        id: String,
        brand: String,
    }

    struct TestUser {
        contact: String,
        password: String,
        captcha: String,
        token: Option<String>,
    }

    struct TestWulianApp2 {
        user: TestUser,
        device: TestDevice,
        wallet: TestWallet,
    }

    struct TestWulianApp {
        user_id: u32,
        token: String,
        contact: String,
        password: String,
        wallets: Vec<TestWallet>,
    }

    async fn init() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse,
            Error = Error,
            InitError = (),
        >,
    > {
        env::set_var("SERVICE_MODE", "test");
        common::log::init_logger();
        models::general::table_all_clear();
        clear_contract().await;
        App::new()
            .configure(configure_routes)
            .configure(crate::account_manager::configure_routes)
    }

    fn simulate_sender_master() -> TestWulianApp2 {
        TestWulianApp2{
            user: TestUser {
                contact: "test000001@gmail.com".to_string(),
                password: "123456789".to_string(),
                captcha: "000001".to_string(),
                token: None,
            },
            device: TestDevice{
                id: "1".to_string(),
                brand: "Apple".to_string(),
            },
            wallet: TestWallet {
                main_account: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
                pubkey: Some("2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string()),
                prikey: Some("8eeb94ead4cf1ebb68a9083c221064d2f7313cd5a70c1ebb44ec31c126f09bc62fa7\
                  ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string()),
                subaccount:vec!["0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9".to_string()], 
                sub_prikey:Some(vec!["2e1eee23ac76477ff1f9e9ae05829b0de3b89072d104c9de6daf0b1c38eddede0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9".to_string()]),
            },
        }
    }

    fn simulate_sender_servant() -> TestWulianApp2 {
        TestWulianApp2 {
            user: TestUser {
                contact: "test000001@gmail.com".to_string(),
                password: "123456789".to_string(),
                captcha: "000001".to_string(),
                token: None,
            },
            device: TestDevice {
                id: "2".to_string(),
                brand: "Apple".to_string(),
            },
            wallet: TestWallet {
                main_account: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0"
                    .to_string(),
                subaccount: vec![
                    "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9".to_string(),
                ],
                sub_prikey: None,
                pubkey: Some(
                    "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e".to_string(),
                ),
                prikey: Some(
                    "2b2193968a4e6ff5c6b8b51f8aed0ee41306c57d225885fca19bbc828a91d1a07d2e\
                7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e"
                        .to_string(),
                ),
            },
        }
    }

    fn simulate_sender_new_device() -> TestWulianApp2 {
        TestWulianApp2 {
            user: TestUser {
                contact: "test000001@gmail.com".to_string(),
                password: "123456789".to_string(),
                captcha: "000001".to_string(),
                token: None,
            },
            device: TestDevice {
                id: "4".to_string(),
                brand: "Apple".to_string(),
            },
            wallet: TestWallet {
                main_account: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0"
                    .to_string(),
                subaccount: vec![
                    "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9".to_string(),
                ],
                sub_prikey: None,
                pubkey: Some(
                    "e48815443073117d29a8fab50c9f3feb80439c196d4d9314400e8e715e231849".to_string(),
                ),
                prikey: Some(
                    "e6913a533e66bbb52ca1f9d773154e608a6a9eacb998b61c0a7592b4b0a130c4\
                    e48815443073117d29a8fab50c9f3feb80439c196d4d9314400e8e715e231849"
                        .to_string(),
                ),
            },
        }
    }


    fn simulate_receiver() -> TestWulianApp2 {
        TestWulianApp2{
            user: TestUser {
                contact: "test000002@gmail.com".to_string(),
                password: "123456789".to_string(),
                captcha: "000002".to_string(),
                token: None,
            },
            device: TestDevice{
                id: "3".to_string(),
                brand: "Huawei".to_string(),
            },
            wallet: TestWallet {
                main_account: "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb".to_string(),
                pubkey: Some("535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb".to_string()),
                prikey: Some("119bef4d830c134a13b2a9661dbcf39fbd628bf216aea43a4b651085df521d525\
                35ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb".to_string()),
                subaccount:vec!["19ae808dec479e1516ffce8ab2a0af4cec430d56f86f70e48f1002b912709f89".to_string()], 
                sub_prikey:Some(vec!["a06d01c1c74f33b4558454dbb863e90995543521fd7fc525432fc58b705f8cef19ae808dec479e1516ffce8ab2a0af4cec430d56f86f70e48f1002b912709f89".to_string()]),
            },
        }
    }

    async fn clear_contract() {
        let cli = blockchain::ContractClient::<MultiSig>::new();
        cli.clear_all().await.unwrap();
        //cli.init_strategy(account_id, account_id.to_owned()).await.unwrap();
        //cli.remove_account_strategy(account_id.to_owned()).await.unwrap();
        //cli.remove_tx_index(1u64).await.unwrap();
    }

    async fn get_tx_status_on_chain(txs_index: Vec<u64>) -> Vec<(u64, bool)> {
        let cli = blockchain::ContractClient::<MultiSig>::new();
        cli.get_tx_state(txs_index).await.unwrap().unwrap()
    }


    #[actix_web::test]
    async fn test_replace_servant() {
        let app = init().await;
        let service = test::init_service(app).await;
        let mut sender_master = simulate_sender_master();
        let mut sender_servant = simulate_sender_servant();
        let mut sender_new_device = simulate_sender_new_device();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        test_add_servant!(service,sender_master,sender_servant);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let payload = json!({
            "mainAccount": sender_master.wallet.main_account,
            "oldServantPubkey": sender_servant.wallet.pubkey.unwrap(),
            "newServantPubkey": sender_new_device.wallet.pubkey.unwrap(),
            "newServantPrikeyEncrypedByPwd": sender_new_device.wallet.prikey.clone().unwrap(),
            "newServantPrikeyEncrypedByAnswer": sender_new_device.wallet.prikey.unwrap(),
            "newDeviceId": sender_new_device.device.id
        });
        //claim
        let url = format!("/wallet/replaceServant");
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            &url,
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        println!("{:?}", res.data);
    }


    

    #[actix_web::test]
    async fn test_new_commer_replace_master() {
        //let newcommer_pubkey = "4b8837f83d6b25118275149cb3cf6c57407cb0f1cb0953b0b6faf3a1f171f15b".to_string();
        let newcommer_pubkey = ed25519_key_gen().1;
        println!("newcommer_pubkey {} ",newcommer_pubkey);
        let app = init().await;
        let service = test::init_service(app).await;
        let mut sender_master = simulate_sender_master();
        let sender_sub_secret = ed25519_key_gen();
        let sender_master_secret = ed25519_key_gen();

        sender_master.wallet = TestWallet{ 
            main_account: sender_master_secret.1.clone(),
             pubkey: Some(sender_master_secret.1.clone()), 
             prikey: Some(sender_master_secret.0.clone()), 
             subaccount: vec![sender_sub_secret.1.clone()], 
             sub_prikey: Some(vec![sender_sub_secret.0.clone()])
        };
        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);

        let payload = json!({
            "newcomerPubkey":  newcommer_pubkey
        });
        //claim
        let url = format!("/wallet/genTxNewcomerReplaceMaster");
        let res: BackendRespond<super::handlers::gen_newcomer_replace_master::GenReplaceKeyInfo> = test_service_call!(
            service,
            "post",
            &url,
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );

        let add_key_sig = blockchain::multi_sig::ed25519_sign_data2(
            sender_master.wallet.prikey.as_ref().unwrap(),
            &res.data.add_key_txid
        );

        let delete_key_sig = blockchain::multi_sig::ed25519_sign_data2(
            &sender_master.wallet.prikey.as_ref().unwrap(),
            &res.data.delete_key_txid
        );
        let payload = json!({
            "newcomerPubkey":  newcommer_pubkey.to_string(),
            "addKeyRaw":  res.data.add_key_raw,
            "deleteKeyRaw":  res.data.delete_key_raw,
            "addKeySig":  add_key_sig,
            "deleteKeySig": delete_key_sig,
            "newcomerPrikeyEncryptedByPwd":  "".to_string(),
            "newcomerPrikeyEncryptedByAnswer":  "".to_string()
        });

        //claim
        println!("{:?}",payload.to_string());
        let url = format!("/wallet/commitTxNewcomerReplaceMaster");
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            &url,
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        println!("{:?}", res.data);
    }

    #[actix_web::test]
    async fn test_get_all() {
        let app = init().await;
        let service = test::init_service(app).await;
        let mut sender_master = simulate_sender_master();
        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);

        //balance
        let url = format!("/wallet/balanceList");
        let res: BackendRespond<Vec<(String,String)>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        let sender_strategy = res.data;
        println!("{:?}", sender_strategy);


        let url = format!("/wallet/getSecret?pubkey={}",sender_master.wallet.pubkey.unwrap());
        let res: BackendRespond<SecretStore> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        println!("res {:?}",res.data);
    }


    #[actix_web::test]
    async fn test_faucet_ok() {
        let app = init().await;
        let service = test::init_service(app).await;
        let mut sender_master = simulate_sender_master();
        test_register!(service, sender_master);
        let url = format!("/wallet/balanceList");
        let res: BackendRespond<Vec<(String,String)>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        let sender_strategy = res.data;
        println!("{:?}", sender_strategy);

        test_create_main_account!(service, sender_master);

        //claim
        let url = format!("/wallet/faucetClaim");
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        println!("{}", res.data);

        //balance
        let url = format!("/wallet/balanceList");
        let res: BackendRespond<Vec<(String,String)>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        let sender_strategy = res.data;
        println!("{:?}", sender_strategy);
    }

    #[actix_web::test]
    async fn test_all_braced_wallet_ok_with_fix_key() {
        let mut sender_master = simulate_sender_master();
        let mut receiver = simulate_receiver();
        let mut sender_servant = simulate_sender_servant();
        test_all_braced_wallet_ok(sender_master,receiver,sender_servant).await;
    }

    #[actix_web::test]
    async fn test_all_braced_wallet_ok_with_new_key() {
        let sender_master_secret = ed25519_key_gen();
        println!("{:?}",sender_master_secret);
        let sender_sub_secret = ed25519_key_gen();
        let sender_servant_secret = ed25519_key_gen();
        let receiver_master_secret = ed25519_key_gen();
        let receiver_sub_secret = ed25519_key_gen();
        let coin_cli = ContractClient::<blockchain::coin::Coin>::new(CoinType::DW20);
        coin_cli.send_coin(&sender_master_secret.1, 13u128).await.unwrap();

        let mut sender_master = simulate_sender_master();
        sender_master.wallet = TestWallet{ 
            main_account: sender_master_secret.1.clone(),
             pubkey: Some(sender_master_secret.1.clone()), 
             prikey: Some(sender_master_secret.0.clone()), 
             subaccount: vec![sender_sub_secret.1.clone()], 
             sub_prikey: Some(vec![sender_sub_secret.0.clone()])
        };

        let mut receiver = simulate_receiver();
        receiver.wallet = TestWallet{ 
            main_account: receiver_master_secret.1.clone(),
             pubkey: Some(receiver_master_secret.1), 
             prikey: Some(receiver_master_secret.0), 
             subaccount: vec![receiver_sub_secret.1], 
             sub_prikey: Some(vec![receiver_sub_secret.0])
        };

        let mut sender_servant = simulate_sender_servant();
        sender_servant.wallet = TestWallet{ 
            main_account: sender_master_secret.1.clone(),
             pubkey: Some(sender_servant_secret.1.clone()), 
             prikey: Some(sender_servant_secret.0.clone()), 
             subaccount: vec![sender_sub_secret.1.clone()], 
             sub_prikey: None
        };
        test_all_braced_wallet_ok(sender_master,receiver,sender_servant).await;
    }


    //#[actix_web::test]
    async fn test_all_braced_wallet_ok(
        mut sender_master:  TestWulianApp2,
        mut receiver: TestWulianApp2,
        mut sender_servant: TestWulianApp2) {
        let app = init().await;
        let service = test::init_service(app).await;
        //let mut sender_master = simulate_sender_master();
        //let mut receiver = simulate_receiver();
        //let mut sender_servant = simulate_sender_servant();
        //init: get token by register or login
        test_register!(service, sender_master);
        test_register!(service, receiver);
        test_login!(service, sender_servant);
        test_create_main_account!(service, receiver);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let get_res = test_get_strategy!(service, receiver);
        let receiver_strategy = get_res.data;

        //step1: new sender main_account
        let payload = json!({
            "masterPubkey":  sender_master.wallet.main_account,
            "masterPrikeyEncryptedByPwd": sender_master.wallet.prikey,
            "masterPrikeyEncryptedByAnswer": sender_master.wallet.prikey,
            "subaccountPubkey":  sender_master.wallet.subaccount.first().unwrap(),
            "subaccountPrikeyEncrypedByPwd": sender_master.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "subaccountPrikeyEncrypedByAnswer": sender_master.wallet.sub_prikey.unwrap().first().unwrap(),
            "signPwdHash": ""
        });
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/createMainAccount",
            Some(payload.to_string()),
            Some(&sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);

        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        //step2: generate servent_key in device which hold master_prikey,and send to server after encrypted
        let payload = json!({
            "mainAccount":  sender_master.wallet.main_account,
            "servantPubkey":  sender_servant.wallet.pubkey.as_ref().unwrap(),
            "servantPrikeyEncrypedByPwd":  sender_servant.wallet.prikey.as_ref().unwrap(),
            "servantPrikeyEncrypedByAnswer":  sender_servant.wallet.prikey.as_ref().unwrap(),
            "holderDeviceId":  sender_servant.device.id,
            "holderDeviceBrand": sender_servant.device.brand,
        });
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/addServant",
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        /*** 
         test removeServant
        let res = test_get_strategy!(service, sender_master);
        let sender_strategy = res.data;
        println!("{},,,{:?}", line!(), sender_strategy);

        let payload = json!({
            "servantPubkey":  sender_servant.wallet.pubkey.as_ref().unwrap(),
        });
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/removeServant",
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let res = test_get_strategy!(service, sender_master);
        let sender_strategy = res.data;
        println!("{},,,{:?}", line!(), sender_strategy);
        return;
        ***/

        //给链上确认一些时间
        //step2.1: sender main_account update strategy
        let payload = json!({
            "accountId":  sender_master.wallet.main_account,
            "deviceId": "1",
            "strategy": [{"min": 1, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200u64, "sigNum": 1}]
        });
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/updateStrategy",
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        //step2.2: check sender's new strategy
        let url = format!(
            "/wallet/getStrategy?accountId={}",
            sender_master.wallet.main_account
        );
        let res: BackendRespond<StrategyData> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_master.user.token.as_ref().unwrap())
        );
        let sender_strategy = res.data;
        println!("{:?}", sender_strategy);

        //step2.3: get message of becoming servant,and save encrypted prikey
        let url = format!("/wallet/searchMessage");
        let res: BackendRespond<Vec<AccountMessage>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_servant.user.token.as_ref().unwrap())
        );
        if let AccountMessage::NewcomerBecameSevant(secret) = res.data.first().unwrap() {
            println!(
                "encrypted_prikey_by_password {:?}",
                secret.encrypted_prikey_by_password
            );
            sender_servant.wallet.prikey = Some(secret.encrypted_prikey_by_password.clone());
            //todo: confirm prikey save
            let payload = json!({
                "servantPubkey":  sender_servant.wallet.pubkey.as_ref().unwrap(),
            });
            let url = format!("/wallet/servantSavedSecret");
            let res: BackendRespond<String> = test_service_call!(
                service,
                "post",
                &url,
                Some(payload.to_string()),
                Some(sender_servant.user.token.as_ref().unwrap())
            );
            assert_eq!(res.status_code, 0);
        }

        //step2.4: get device list
        let url = format!("/wallet/deviceList");
        let res: BackendRespond<Vec<DeviceInfo>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(sender_servant.user.token.as_ref().unwrap())
        );
        println!("{},,,{:?}", line!(), res.data);

        //step2.5: add subaccount
        let payload = json!({
            "mainAccount":  sender_master.wallet.main_account,
            "subaccountPubkey": "11111142a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9",
            "subaccountPrikeyEncrypedByPwd": "by_pwd_ead4cf1",
            "subaccountPrikeyEncrypedByAnswer": "byanswer_ead4cf1e",
        });
        let url = format!("/wallet/addSubaccount");
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            &url,
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        let res = test_get_strategy!(service, sender_master);
        let sender_strategy = res.data;
        println!("{},,,{:?}", line!(), sender_strategy);

        //step3: master: pre_send_money
        let payload = json!({
             "from": &sender_master.wallet.main_account,
             "to": &receiver.wallet.main_account,
             "coin":"DW20",
             "amount": 12,
             "expireAt": 1808015513000u64
        });
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/preSendMoney",
            Some(payload.to_string()),
            Some(sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);

        //step3.1: 对于created状态的交易来说，主设备不处理，从设备上传签名
        let res = test_search_message!(service, sender_master);
        if let AccountMessage::CoinTx(_index, tx) = res.data.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "this device hold  master key,and do nothing for 'Created' tx"
            );
        }
        let res = test_search_message!(service, sender_servant);
        if let AccountMessage::CoinTx(index, tx) = res.data.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::Created);
            assert_eq!(
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                sender_strategy.servant_pubkeys.first().unwrap(),
                "this device hold  servant key,need to sig for 'Created' tx"
            );

            //step4: upload sender servant sign
            //local sign
            let signature = blockchain::multi_sig::ed25519_sign_data2(
                &sender_servant.wallet.prikey.unwrap(),
                &tx.coin_tx_raw,
            );
            let signature = format!(
                "{}{}",
                sender_servant.wallet.pubkey.as_ref().unwrap(),
                signature
            );

            //upload_servant_sig
            let payload = json!({
                "txIndex": index,
                "signature": signature,
            });
            let res: BackendRespond<String> = test_service_call!(
                service,
                "post",
                "/wallet/uploadServantSig",
                Some(payload.to_string()),
                Some(sender_servant.user.token.as_ref().unwrap())
            );
            assert_eq!(res.status_code, 0);
        }

        //step5: receiver get notice and react it
        let res = test_search_message!(service, receiver);
        if let AccountMessage::CoinTx(index, tx) = res.data.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::SenderSigCompleted);
            assert_eq!(
                receiver.wallet.pubkey.unwrap(),
                receiver_strategy.master_pubkey,
                "only master_key can ratify or refuse it"
            );

            let payload = json!({
                "txIndex": index,
                "isAgreed": true,
            });
            let res: BackendRespond<String> = test_service_call!(
                service,
                "post",
                "/wallet/reactPreSendMoney",
                Some(payload.to_string()),
                Some(receiver.user.token.as_ref().unwrap())
            );
            assert_eq!(res.status_code, 0);
        }

        //step6: sender_master get notice and react it
        //todo: 为了减少一个接口以及减掉客户端交易组装的逻辑，在to账户确认的时候就生成了txid和raw_data,所以master只有1分钟的确认时间
        //超过了就链上过期（非多签业务过期）
        let res = test_search_message!(service, sender_master);
        if let AccountMessage::CoinTx(index, tx) = res.data.first().unwrap() {
            assert_eq!(tx.status, CoinTxStatus::ReceiverApproved);
            assert_eq!(
                sender_master.wallet.pubkey.as_ref().unwrap(),
                &sender_strategy.master_pubkey,
                "only sender_master_key can reconfirm or refuse it"
            );

            //local sign
            let signature = blockchain::multi_sig::ed25519_sign_data2(
                sender_master.wallet.prikey.as_ref().unwrap(),
                tx.tx_id.as_ref().unwrap(),
            );

            let payload = json!({
                "txIndex": index,
                "confirmedSig": signature
            });

            let res: BackendRespond<String> = test_service_call!(
                service,
                "post",
                "/wallet/reconfirmSendMoney",
                Some(payload.to_string()),
                Some(sender_master.user.token.as_ref().unwrap())
            );
            assert_eq!(res.status_code, 0);
        }

        let txs_success = get_tx_status_on_chain(vec![1u64, 2u64]).await;
        println!("txs_success {:?}", txs_success);
    }
}
