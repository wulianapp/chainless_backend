//! account manager http service

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

mod transaction;

use actix_cors::Cors;
use actix_web::{
    error, get, http, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use std::env;

use actix_web_httpauth::middleware::HttpAuthentication;
use blockchain::coin::decode_coin_transfer;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus};
use common::error_code::{AccountManagerError, WalletError};
use common::token_auth;
use log::info;
use models::account_manager;
use models::account_manager::UserFilter;
use models::coin_transfer::CoinTxFilter;
use models::wallet::{get_wallet, WalletFilter};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
//use crate::transaction::{get_all_message, get_user_message, insert_new_message, MessageType, update_message_status};

#[derive(Deserialize, Serialize)]
struct BackendRespond<T: Serialize> {
    status_code: u16,
    msg: String,
    //200 default success
    data: T,
}

fn generate_ok_respond(info: Option<impl Serialize>) -> HttpResponse {
    if let Some(data) = info {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully".to_string(),
            status_code: 0u16,
            data,
        })
    } else {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully".to_string(),
            status_code: 0u16,
            data: "".to_string(),
        })
    }
}

fn generate_error_respond(error: WalletError) -> HttpResponse {
    return HttpResponse::Ok().json(BackendRespond {
        msg: error.to_string(),
        status_code: error.code(),
        data: "".to_string(),
    });
}

pub fn gen_extra_respond(inner_res: Result<Option<impl Serialize>, WalletError>) -> impl Responder {
    match inner_res {
        Ok(data) => generate_ok_respond(data),
        Err(error) => {
            if error.to_string().contains("Authorization") {
                HttpResponse::Unauthorized().json(error.to_string())
            } else {
                generate_error_respond(error)
            }
        }
    }
}


/**
 * @api {post} /wallet/searchMessage  notify user tx request
 * @apiVersion 0.0.1
 * @apiName searchMessage
 * @apiGroup Wallet
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8065/wallet/searchMessage
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
 *   OiJIUzI1NiJ9.eyJzdWIiOiJjaGFpbmxlc3MgdXNlcmlkOiAgNCIsImlhdCI6MTcwMzk1Njk5NywiZXhw
 * IjoxNzA1MjUyOTk3fQ.usNdrNVo2oMO0rMdW62rbbooxzOKZjoji9cNN2b1I1c'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8069/wallet/searchMessage
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchMessageRequest {
    //unused
    user_id: String,
}

async fn search_message_inner(req: HttpRequest) -> Result<Option<impl Serialize>, WalletError> {
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    println!("searchMessage user_id {}", user_id);
    let message = models::coin_transfer::get_transactions(CoinTxFilter::ByUserPending(user_id));
    Ok(Some(message))
}

#[post("/wallet/searchMessage")]
async fn search_message(req: HttpRequest) -> impl Responder {
    gen_extra_respond(search_message_inner(req).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    tx_raw: String,
    //platform_sign_num: u8
}

async fn pre_send_money_inner(
    req: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> Result<Option<impl Serialize>, WalletError> {
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    let mut coin_tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw).unwrap();
    coin_tx.status = CoinTxStatus::Created;
    if coin_tx.sender != user_id {
        Err(WalletError::TxFromNotBeUser)?;
    }

    //for receiver
    if let Some(user) = account_manager::get_by_user(UserFilter::ById(coin_tx.receiver)) {
        let _tx = models::coin_transfer::single_insert(&coin_tx).unwrap();
    } else {
        Err(WalletError::ReceiverNotFound)?;
    }
    Ok(None::<String>)
}

#[post("/wallet/preSendMoney")]
async fn pre_send_money(
    req: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(pre_send_money_inner(req, request_data).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReactPreSendMoney {
    tx_id: String,
    is_agreed: bool,
}
async fn react_pre_send_money_inner(
    req: HttpRequest,
    request_data: web::Json<ReactPreSendMoney>,
) -> Result<Option<impl Serialize>, WalletError> {
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    let ReactPreSendMoney { tx_id, is_agreed } = request_data.0;
    //message max is 10，
    //let FinalizeSha = request_data.clone();
    let status = if is_agreed {
        CoinTxStatus::ReceiverApproved
    } else {
        CoinTxStatus::ReceiverRejected
    };
    models::coin_transfer::update_status(status, CoinTxFilter::ByTxId(tx_id));
    Ok(None::<String>)
}

#[post("/wallet/reactPreSendMoney")]
async fn react_pre_send_money(
    req: HttpRequest,
    request_data: web::Json<ReactPreSendMoney>,
) -> impl Responder {
    gen_extra_respond(react_pre_send_money_inner(req, request_data).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconfirmSendMoneyRequest {
    device_id: String,
    tx_id: String,
    is_confirmed: bool,
}
async fn reconfirm_send_money_inner(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> Result<Option<impl Serialize>, WalletError> {
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    //todo: check must be main device
    let ReconfirmSendMoneyRequest {
        device_id,
        tx_id,
        is_confirmed,
    } = request_data.0;

    let status = if is_confirmed {
        CoinTxStatus::SenderReconfirmed
    } else {
        CoinTxStatus::SenderCanceled
    };
    models::coin_transfer::update_status(status, CoinTxFilter::ByTxId(tx_id));
    Ok(None::<String>)
}

#[post("/wallet/reconfirmSendMoney")]
async fn send_money(
    req: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(reconfirm_send_money_inner(req, request_data).await)
}


/**
 * @api {post} /wallet/uploadTxSignature  upload tx signatures
 * @apiVersion 0.0.1
 * @apiName uploadTxSignature
 * @apiGroup Wallet
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} txId transaction_id of chain
 * @apiBody {String} signature  data of every key of multi-account sign the same raw_transaction
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8065/wallet/uploadTxSignature
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
 *   OiJIUzI1NiJ9.eyJzdWIiOiJjaGFpbmxlc3MgdXNlcmlkOiAgNCIsImlhdCI6MTcwMzk1Njk5NywiZXhw
 * IjoxNzA1MjUyOTk3fQ.usNdrNVo2oMO0rMdW62rbbooxzOKZjoji9cNN2b1I1c'
 * -d'{"deviceId": "00001","txId": "abc123","signature":"abc123"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8069/wallet/uploadTxSignature
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct uploadTxSignatureRequest {
    device_id: String,
    tx_id: String,
    signature: String,
}

async fn upload_tx_signed_data_inner(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> Result<Option<impl Serialize>, WalletError> {
    //todo: check tx_status must be SenderReconfirmed
    //todo:check user_id if valid
    let user_id =
        token_auth::validate_credentials(&req).map_err(|e| WalletError::Authorization(e))?;

    //todo: check must be main device
    let uploadTxSignatureRequest {
        device_id,
        tx_id,
        signature,
    } = request_data.0;

    //todo: validate signature

    let tx = models::coin_transfer::get_transactions(CoinTxFilter::ByTxId(tx_id.clone()));
    let mut signatures = tx.first().unwrap().transaction.signatures.clone();
    signatures.push(signature);
    models::coin_transfer::update_signature(signatures, CoinTxFilter::ByTxId(tx_id.clone()));
    //todo: collect enough signatures
    let wallet_info = get_wallet(WalletFilter::ByUserId(user_id));
    let wallet_info = &wallet_info.first().unwrap().wallet;

    if wallet_info.sign_strategies.len() == 1
        && *wallet_info.sign_strategies.first().unwrap() == "1-1".to_string()
    //todo: check sign strategy if ok,broadcast this tx
    {
        //broadcast(signatures)
        models::coin_transfer::update_status(CoinTxStatus::Broadcast, CoinTxFilter::ByTxId(tx_id));
    }
    Ok(None::<String>)
}
#[post("/wallet/uploadTxSignature")]
async fn upload_tx_signed_data(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> impl Responder {
    gen_extra_respond(upload_tx_signed_data_inner(req, request_data).await)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let global_conf = &common::env::CONF;
    let service = format!("0.0.0.0:{}", global_conf.wallet_api_port);

    HttpServer::new(move || {
        //let auth = HttpAuthentication::bearer(token_auth::validate_credentials);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    //.allowed_origin("127.0.0.1")
                    //.send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .service(search_message)
            .service(pre_send_money)
            .service(react_pre_send_money)
            .service(send_money)
            .service(upload_tx_signed_data)
    })
    .bind(service.as_str())?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_http::Request;
    use actix_web::body::MessageBody;
    use actix_web::dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::middleware::ErrorHandlerResponse::Response;
    use actix_web::{body, body::MessageBody as _, rt::pin, test, web, App};
    use common::data_structures::wallet::Wallet;
    use models::coin_transfer::CoinTxView;
    use models::wallet;
    use serde_json::json;

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
        models::general::table_all_clear();
        App::new()
            .service(search_message)
            .service(pre_send_money)
            .service(react_pre_send_money)
            .service(upload_tx_signed_data)
    }

    fn simulate_sender() -> (String, u32) {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjEs\
        ImlhdCI6MTcwNDc5NjQ0OTc2NywiZXhwIjoxNzA2MDkyNDQ5NzY3fQ.OOlutDTr-hWTf6m\
        smbHSmfG0kiwnAI6HVMnGU19hOtQ"
            .to_string();
        let user_id = 1u32;
        let mut user_info = UserInfo::default();
        user_info.email = "test1@gmail.com".to_string();
        user_info.pwd_hash = "123456789".to_string();
        user_info.multi_sign_strategy = "2-2".to_string();
        user_info.invite_code = user_id.to_string();
        account_manager::single_insert(user_info).unwrap();

        let wallet = Wallet {
            user_id,
            account_id: "1".to_string(),
            sub_pubkeys: vec!["1".to_string()],
            sign_strategies: vec!["1-1".to_string()],
            participate_device_ids: vec!["1".to_string()],
        };
        models::wallet::single_insert(&wallet).unwrap();

        (token, user_id)
    }

    fn simulate_receiver() -> (String, u32) {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdW\
        IiOjIsImlhdCI6MTcwNDg1MzQ5MjI3OCwiZXhwIjoxNzA2MTQ5NDkyMjc4fQ.CK-F\
        f-350XzW1SoAfLtJtcxqvFE7_7Z16zPrSJzh7Fc"
            .to_string();
        let user_id = 2u32;
        let mut user_info = UserInfo::default();
        user_info.email = "test2@gmail.com".to_string();
        user_info.pwd_hash = "123456789".to_string();
        user_info.multi_sign_strategy = "2-2".to_string();
        user_info.invite_code = user_id.to_string();
        account_manager::single_insert(user_info).unwrap();
        (token, user_id)
    }

    #[actix_web::test]
    async fn test_all_braced_wallet_ok() {
        //1.node0 send 10 cly to 2.node0
        let payload = r#"{
            "txRaw": "07000000312e6e6f646530001409d2c60903529a8c8ca617abe04151de045632ae4181b8099f8f97153000f101e6008dfd01000009000000636c792e6e6f646530432b91c327c1ee6bacb855c011f2c8649d1bec5d0e3cb91efeafec024cf8405c01000000020b00000066745f7472616e73666572320000007b22616d6f756e74223a3132332c226d656d6f223a6e756c6c2c2272656365697665725f6964223a22322e6e6f646530227d00407a10f35a000000000000000000000000000000000000"
        }
        "#;

        let app = init().await;
        let service = test::init_service(app).await;
        let sender = simulate_sender();
        let receiver = simulate_receiver();

        //step 01: send transfer reuqest
        let req = test::TestRequest::post()
            .uri("/wallet/preSendMoney")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", sender.0)))
            .set_payload(payload.to_string())
            .to_request();

        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<String> = serde_json::from_str(body_str.as_str()).unwrap();
        println!("sender preSendMoneyRes {}", user.data);
        //for receiver after preSendMoney
        let req = test::TestRequest::post()
            .uri("/wallet/searchMessage")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", receiver.0)))
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("line {}:,{}", line!(), body_str);
        let user: BackendRespond<Vec<CoinTxView>> =
            serde_json::from_str(body_str.as_str()).unwrap();
        println!("receiver searchMessageRes {:?}", user.data);
        assert_eq!(
            user.data.first().unwrap().transaction.status,
            CoinTxStatus::Created
        );

        //step 02: receiver approve tx
        let payload = json!({
            "txId": user.data.first().unwrap().transaction.tx_id,
            "isAgreed": true,
        });
        let req = test::TestRequest::post()
            .uri("/wallet/reactPreSendMoney")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", receiver.0)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("line {}:,{}", line!(), body_str);
        let user: BackendRespond<String> = serde_json::from_str(body_str.as_str()).unwrap();
        println!("receiver reactPreSendMoney res {:?}", user.data);
        //for sender after react preSendMoney
        let req = test::TestRequest::post()
            .uri("/wallet/searchMessage")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", sender.0)))
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<Vec<CoinTxView>> =
            serde_json::from_str(body_str.as_str()).unwrap();
        println!("sender searchMessageRes {:?}", user.data);
        assert_eq!(
            user.data.first().unwrap().transaction.status,
            CoinTxStatus::ReceiverApproved
        );

        //step 03: for all sender devices upload_tx_signed_data_inner
        let payload = json!({
            "txId": user.data.first().unwrap().transaction.tx_id,
            "deviceId": "111",
            "signature": "2222"
        });
        let req = test::TestRequest::post()
            .uri("/wallet/uploadTxSignature")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", sender.0)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<String> = serde_json::from_str(body_str.as_str()).unwrap();
        //for sender after react uploadTxSignature,have a msg for to multiple sign
        let req = test::TestRequest::post()
            .uri("/wallet/searchMessage")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", sender.0)))
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<Vec<CoinTxView>> =
            serde_json::from_str(body_str.as_str()).unwrap();
        println!("sender searchMessageRes {:?}", user.data);
        assert_eq!(user.data.len(), 0);
    }
}
