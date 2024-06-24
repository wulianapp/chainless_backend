//! account manager http service

pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};

use handlers::add_servant::AddServantRequest;
use handlers::add_subaccount::AddSubaccountRequest;
use handlers::balance_list::BalanceListRequest;
use handlers::cancel_send_money::CancelSendMoneyRequest;
use handlers::commit_newcomer_replace_master::CommitNewcomerSwitchMasterRequest;
use handlers::commit_servant_switch_master::CommitServantSwitchMasterRequest;
use handlers::create_main_account::CreateMainAccountRequest;
use handlers::estimate_transfer_fee::EstimateTransferFeeRequest;
use handlers::faucet_claim::FaucetClaimRequest;
use handlers::gen_newcomer_switch_master::GenNewcomerSwitchMasterRequest;
use handlers::gen_send_money::GenSendMoneyRequest;
use handlers::gen_servant_switch_master::GenServantSwitchMasterRequest;
use handlers::get_need_sig_num::GetNeedSigNumRequest;
use handlers::get_tx::GetTxRequest;
use handlers::newcommer_switch_servant::NewcommerSwitchServantRequest;
use handlers::pre_send_money::PreSendMoneyRequest;
use handlers::pre_send_money_to_sub::PreSendMoneyToSubRequest;
use handlers::react_pre_send_money::ReactPreSendMoneyRequest;
use handlers::reconfirm_send_money::ReconfirmSendMoneyRequest;
use handlers::remove_servant::RemoveServantRequest;
use handlers::remove_subaccount::RemoveSubaccountRequest;
use handlers::single_balance::SingleBalanceRequest;
use handlers::sub_send_to_main::SubSendToMainRequest;
use handlers::tx_list::TxListRequest;
use handlers::update_security::UpdateSecurityRequest;
use handlers::update_strategy::UpdateStrategyRequest;
use handlers::update_subaccount_hold_limit::UpdateSubaccountHoldLimitRequest;
use handlers::upload_servant_sig::UploadTxSignatureRequest;

use handlers::get_secret::GetSecretRequest;
use handlers::set_fees_priority::SetFeesPriorityRequest;

use crate::utils::respond::gen_extra_respond;
//use crate::transaction::{get_all_message, get_user_message, insert_new_message, MessageType, update_message_status};

use tracing::debug;

use crate::utils::respond::get_lang;

use crate::utils::respond::get_trace_id;

/**
* @api {get} /wallet_v2/searchMessage 查询待处理的钱包消息
* @apiVersion 0.0.2
* @apiName searchMessage
* @apiGroup Wallet
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet_v2/searchMessage
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1} status_code         状态码.
* @apiSuccess {String} msg 状态信息                 description of status.
* @apiSuccess {object} data                当前需要处理的消息详情.
* @apiSuccess {object[]} data.newcomer_became_sevant    新设备成为从设备消息
* @apiSuccess {String} data.newcomer_became_sevant.pubkey           被分配的servant_pubkey
* @apiSuccess {String} data.newcomer_became_sevant.state            不用关注
* @apiSuccess {Number} data.newcomer_became_sevant.user_id          所属用户id
* @apiSuccess {String} data.newcomer_became_sevant.encrypted_prikey_by_password    安全密码加密私钥的输出
* @apiSuccess {String} data.newcomer_became_sevant.encrypted_prikey_by_answer      安全问答加密私钥的输出
* @apiSuccess {Bool} data.have_pending_txs             作为发送方是否有待处理的交易
* @apiSuccess {object[]} data.coin_tx                转账消息
* @apiSuccess {Number} data.coin_tx.order_id          交易订单号.
* @apiSuccess {object} data.coin_tx.transaction        交易详情.
* @apiSuccess {String} [data.coin_tx.transaction.tx_id]        链上交易id.
* @apiSuccess {String=BTC,ETH,USDT,USDC,DW20,CLY} data.coin_tx.transaction.coin_type      币种名字
* @apiSuccess {String} data.coin_tx.transaction.from                发起方
* @apiSuccess {String} data.coin_tx.transaction.to                接收方
* @apiSuccess {String} data.coin_tx.transaction.amount               交易量
* @apiSuccess {String} data.coin_tx.transaction.expireAt             交易截止时间戳
* @apiSuccess {String} [data.coin_tx.transaction.memo]                交易备注
* @apiSuccess {String=
    Created(转账订单创建),
    SenderSigCompleted（发起方从设备收集到足够签名）,
    ReceiverApproved   （接受者接受转账）,
    ReceiverRejected    （接受者拒绝收款）,
    SenderCanceled      （发送者取消发送）,
    SenderReconfirmed   （发送者确认发送）
} data.CoinTx.transaction.stage                交易进度
* @apiSuccess {String}  data.coin_tx.transaction.coin_tx_raw       币种转账的业务原始数据hex
* @apiSuccess {String} [data.coin_tx.transaction.chain_tx_raw]          链上交互的原始数据
* @apiSuccess {String[]} data.coin_tx.transaction.signatures         从设备对业务数据的签名
* @apiSuccess {String=
    NotLaunch(未上链),
    Pending(上链待确认),
    Failed(上链但执行失败),
    Successful(上链确认成功),
}  data.coin_tx.transaction.chain_status        交易的链上状态
* @apiSuccess {String=
    Normal(普通转账),
    Forced(强制转账),
    MainToSub(当前用户的主账户给子账户转账),
    SubToMain(当前用户的子账户给主账户转账),
    MainToBridge(跨链转出)
} data.coin_tx.transaction.tx_type         从设备对业务数据的签名
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/searchMessage
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/wallet_v2/searchMessage")]
async fn search_message(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::search_message::req(req).await)
}

/**
* @api {get} /wallet_v2/getSecret 获取备份密钥信息
* @apiVersion 0.0.2
* @apiName GetSecret
* @apiGroup Wallet
* @apiQuery {String=All,Single}  type  请求的类型:All获取主、从、子所有
* @apiQuery {String}  [accountId]      钱包id，如果type为All则该字段被无视，single的时候，
    如果不传则返回当前设备所属的所有密钥，
    从设备返回主账户servent的密钥
    主设备返回主账户master（index为0）和所有子账户的私钥
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet_v2/getSecret
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3011,3004} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {Object[]} data                                    备份加密密钥信息.
* @apiSuccess {String} data.pubkey                             对应的公钥
* @apiSuccess {String} data.state                              不关注
* @apiSuccess {Number} data.user_id                            所属用户ID
* @apiSuccess {String} data.encrypted_prikey_by_password       被安全密码加密后的文本
* @apiSuccess {String} data.encrypted_prikey_by_answer         被安全问答加密后的文本
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/getSecret
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/wallet_v2/getSecret")]
async fn get_secret(
    req: HttpRequest,
    request_data: web::Query<GetSecretRequest>,
) -> impl Responder {
    debug!(
        "req_params:: {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::get_secret::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet_v2/preSendMoney 主钱包发起预交易
 * @apiVersion 0.0.2
 * @apiName preSendMoney
 * @apiGroup Wallet
 * @apiBody {String} from    发起方多签钱包ID
 * @apiBody {String} to      收款方ID（可以是手机、邮箱、钱包id）
 * @apiBody {String=BTC,ETH,USDT,USDC,DW20,CLY} coin      币种名字
 * @apiBody {String} amount      转账数量
 * @apiBody {Number} expireAt      有效截止时间戳
 * @apiBody {String} [memo]      交易备注
 * @apiBody {String} isForced      是否强制交易
 * @apiBody {String} [captcha]      如果是无需从设备签名的交易，则需要验证码
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/preSendMoney
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
* @apiSuccess {String=0,1,3008,3017,3010,3018} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {object} data                     订单结果.
* @apiSuccess {Number} data.0                交易序列号.
* @apiSuccess {String} [data.1]                待签名数据(txid).
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/preSendMoney
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/preSendMoney")]
async fn pre_send_money(
    req: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    debug!(
        "req_params:: {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::pre_send_money::req(req, request_data.0).await,
    )
}

/**
 * @api {post} /wallet_v2/reactPreSendMoney 接受者收款确认
 * @apiVersion 0.0.2
 * @apiName reactPreSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} orderId    交易订单号
 * @apiBody {bool} isAgreed    是否同意接收
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/reactPreSendMoney
   -d ' {
        "orderId": 1,
        "isAgreed": true
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3008,3013} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/reactPreSendMoney
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/reactPreSendMoney")]
async fn react_pre_send_money(
    req: HttpRequest,
    request_data: web::Json<ReactPreSendMoneyRequest>,
) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::react_pre_send_money::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet_v2/reconfirmSendMoney 发起方打款二次确认
 * @apiVersion 0.0.2
 * @apiName reconfirmSendMoney
 * @apiGroup Wallet
 * @apiBody {Number} orderId    交易订单号
 * @apiBody {String} confirmedSig    再确认就传签名结果,取消的话用cancelSendMoney的接口
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/reconfirmSendMoney
   -d ' {
        "deviceId":  "1",
        "orderId": 1,
        "confirmedSig": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e6
                     83ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4e4c12e677ce
                35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600"
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3008,3011} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/reconfirmSendMoney
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/reconfirmSendMoney")]
async fn reconfirm_send_money(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::reconfirm_send_money::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet_v2/cancelSendMoney   发起方取消交易
 * @apiVersion 0.0.2
 * @apiName CancelSendMoney
 * @apiGroup Wallet
 * @apiBody {String} orderId    交易订单号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/cancelSendMoney
   -d ' {
        "deviceId":  "1",
        "orderId": 1,
        "confirmedSig": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e6
                     83ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4e4c12e677ce
                35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600"
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3008,3011,3013} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/canceSendMoney
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/cancelSendMoney")]
async fn cancel_send_money(
    req: HttpRequest,
    request_data: web::Json<CancelSendMoneyRequest>,
) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::cancel_send_money::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /wallet_v2/uploadServantSig 上传从密钥的多签签名
 * @apiVersion 0.0.2
 * @apiName UsploadServantSig
 * @apiGroup Wallet
 * @apiBody {Number} orderId    交易订单号
 * @apiBody {String} signature  pubkey和签名结果的拼接
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/uploadServantSig
   -d ' {
        "orderId": 1,
        "signature": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe
3ac02e2bff9ee3e683ccf89e6a345b853fa985b9ec860b913616e3a9f7edd418a224f569e4
e4c12e677ce35b7e61c0b2b67907befd3b0939ed6c5f4a9fc0c9666b011b9050d4600",
           }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3008,3011} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                null
 * @apiSampleRequest http://120.232.251.101:8066/wallet_v2/uploadServantSig
 */

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/uploadServantSig")]
async fn upload_servant_sig(
    req: HttpRequest,
    request_data: web::Json<UploadTxSignatureRequest>,
) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::upload_servant_sig::req(req, request_data.into_inner()).await,
    )
}

/***
* @api {post} /wallet_v2/servantSavedSecret 从设备告知服务端密钥已保存
* @apiVersion 0.0.2
* @apiName ServantSavedSecret
* @apiGroup Wallet
* @apiBody {String} servantPubkey   从公钥
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet_v2/servantSavedSecret
  -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/servantSavedSecret
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/servantSavedSecret")]
async fn servant_saved_secret(req: HttpRequest) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::servant_saved_secret::req(req).await,
    )
}


/**
 * @api {get} /wallet_v2/txList 账单列表详情
 * @apiVersion 0.0.2
 * @apiName TxList
 * @apiGroup Wallet
 * @apiQuery {String=Sender,Receiver} TransferRole  交易中的角色，对应ui的转出和收款栏
 * @apiQuery {String}                 [counterparty]   交易对手方
 * @apiQuery {Number}                 perPage           每页的数量
 * @apiQuery {Number}                 page            页数的序列号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/txList?
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {object[]} data                交易详情数组.
* @apiSuccess {Number} data.order_id          交易索引号.
* @apiSuccess {object} data.transaction        交易详情.
* @apiSuccess {String} [data.tx_id]        链上交易id.
* @apiSuccess {String=BTC,ETH,USDT,USDC,CLY,DW20} data.coin_type      币种名字
* @apiSuccess {String} data.from                发起方
* @apiSuccess {String} data.to                接收方
* @apiSuccess {String} data.amount               交易量
* @apiSuccess {String} data.CoinTx.transaction.expireAt             交易截止时间戳
* @apiSuccess {String} [data.memo]                交易备注
* @apiSuccess {String=
    Created(转账订单创建),
    SenderSigCompleted(发起方从设备收集到足够签名),
    ReceiverApproved(接受者接受转账),
    ReceiverRejected(接受者拒绝收款),
    SenderCanceled(发送者取消发送),
    SenderReconfirmed(发送者确认发送）
} data.CoinTx.transaction.stage                交易进度
* @apiSuccess {String}  data.coin_tx_raw       币种转账的业务原始数据hex
* @apiSuccess {String} [data.chain_tx_raw]          链上交互的原始数据
* @apiSuccess {String=
    NotLaunch(未上链),
    Pending(上链待确认),
    Failed(上链但执行失败),
    Successful(上链确认成功),
}  data.coin_tx.transaction.chain_status        交易的链上状态
* @apiSuccess {String=
    Normal(普通转账),
    Forced(强制转账),
    MainToSub(当前用户的主账户给子账户转账),
    SubToMain(当前用户的子账户给主账户转账),
    MainToBridge(跨链转出)
} data.coin_tx.transaction.tx_type         从设备对业务数据的签名
* @apiSuccess {object[]} data.signatures       从设备签名详情
* @apiSuccess {String} data.signatures.pubkey         签名公钥
* @apiSuccess {String} data.signatures.sig            签名结果
* @apiSuccess {String} data.signatures.device_id      签名设备id
* @apiSuccess {String} data.signatures.device_brand   签名设备品牌
* @apiSuccess {String} data.updated_at         交易更新时间戳
* @apiSuccess {String} data.created_at         交易创建时间戳
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/balanceList
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/wallet_v2/txList")]
async fn tx_list(req: HttpRequest, request_data: web::Query<TxListRequest>) -> impl Responder {
    debug!(
        "req_params::  {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::tx_list::req(req, request_data.0).await,
    )
}

/**
 * @api {get} /wallet_v2/getTx 单个账单详情
 * @apiVersion 0.0.2
 * @apiName GetTx
 * @apiGroup Wallet
 * @apiQuery {String}                 orderId            交易订单号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/getTx?index=1
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,2,3021} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {object} data               交易详情.
* @apiSuccess {String} data.order_id          交易id号.
* @apiSuccess {object} data.transaction        交易详情.
* @apiSuccess {String} [data.tx_id]        链上交易id.
* @apiSuccess {String=BTC,ETH,USDT,USDC,CLY,DW20} data.coin_type      币种名字
* @apiSuccess {String} data.from                发起方
* @apiSuccess {String} data.to                接收方的联系方式或钱包id
* @apiSuccess {String} data.to_account_id                接收方的钱包id
* @apiSuccess {String} data.amount               交易量
* @apiSuccess {String} data.expire_at             交易截止时间戳
* @apiSuccess {String} [data.memo]                交易备注
* @apiSuccess {String=
    Created(转账订单创建),
    SenderSigCompleted（发起方从设备收集到足够签名）,
    ReceiverApproved（接受者接受转账）,
    ReceiverRejected（接受者拒绝收款）,
    SenderCanceled（发送者取消发送）,
    SenderReconfirmed（发送者确认发送）
} data.stage
    交易进度分别对应{转账订单创建、从设备签名准备完毕、接收者同意收款、接收者拒绝收款、发送方取消转账、发送方二次确认交易}
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
* @apiSuccess {String=Normal,Forced,MainToSub,SubToMain,MainToBridge} data.coin_tx.transaction.tx_type         从设备对业务数据的签名
* @apiSuccess {String=NotLaunch(未上链),
    Pending(待确认),
    Failed(失败),
    Successful(成功)}  data.chain_status       交易的链上状态
* @apiSuccess {Object[]}  data.fees_detail           手续费详情,Successful状态为实际否则为预估
* @apiSuccess {String=BTC,ETH,USDT,USDC,DW20}  data.fees_detail.fee_coin           手续费币种
* @apiSuccess {String}  data.fees_detail.fee_amount         手续费数量
* @apiSuccess {String} data.updated_at         交易更新时间戳
* @apiSuccess {String} data.created_at         交易创建时间戳
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/getTx
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/wallet_v2/getTx")]
async fn get_tx(req: HttpRequest, request_data: web::Query<GetTxRequest>) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::get_tx::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {get} /wallet_v2/deviceList 返回设备信息列表
 * @apiVersion 0.0.2
 * @apiName deviceList
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/deviceList
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {object[]} data                 设备信息列表.
* @apiSuccess {String} data.id                设备id.
* @apiSuccess {String} data.user_id           设备当前所属用户id.
* @apiSuccess {String} data.state             （不关注）.
* @apiSuccess {String} data.brand             设备品牌.
* @apiSuccess {String} [data.hold_pubkey]            设备持有的pubkey,未成为master或者servant之前为空
* @apiSuccess {String} data.holder_confirm_saved   设备主钱包持有的master或者servant的pubkey.
* @apiSuccess {String=Master,Servant,Undefined} data.key_role           当前设备持有的key的类型

* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/deviceList
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/wallet_v2/deviceList")]
async fn device_list(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::device_list::req(req).await)
}



/**
 * @api {post} /wallet_v2/getNeedSigNum   获取转账需要几个从设备签名
 * @apiVersion 0.0.2
 * @apiName GetNeedSigNum
 * @apiGroup Wallet
 * @apiBody {String}     coin                  转账币种
 * @apiBody {String}     amount                转账数量
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/wallet_v2/genServantSwitchMaster
   -d '  {
             "encryptedPrikey": "",
             "pubkey": "",
            }'
   -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
    OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
    iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3007} status_code         状态码.
* @apiSuccess {String=HaveUncompleteTx} msg
* @apiSuccess {Number} data                所需签名数量
* @apiSampleRequest http://120.232.251.101:8066/wallet_v2/getNeedSigNum
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/wallet_v2/getNeedSigNum")]
async fn get_need_sig_num(
    req: HttpRequest,
    request_data: web::Json<GetNeedSigNumRequest>,
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::get_need_sig_num::req(req, request_data.into_inner()).await,
    )
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_message)
        .service(pre_send_money)
        .service(react_pre_send_money)
        .service(reconfirm_send_money)
        .service(upload_servant_sig)
        .service(servant_saved_secret)
        .service(device_list)
        .service(get_secret)
        .service(tx_list)
        .service(get_tx)
        .service(cancel_send_money)
        .service(get_need_sig_num);
    //.service(remove_subaccount);
}