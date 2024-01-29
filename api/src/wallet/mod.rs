//! account manager http service

mod transaction;
mod handlers;

use actix_cors::Cors;
use actix_web::{
    error, get, http, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use std::env;

use actix_web_httpauth::middleware::HttpAuthentication;
use blockchain::coin::decode_coin_transfer;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, SecretKeyType};
use common::error_code::{AccountManagerError, WalletError};
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
use lettre::transport::smtp::client::CertificateStore::Default;
use common::http::gen_extra_respond;
//use crate::transaction::{get_all_message, get_user_message, insert_new_message, MessageType, update_message_status};

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

#[post("/wallet/searchMessage")]
async fn search_message(request: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::search_message::req(request).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    tx_raw: String,
    //platform_sign_num: u8
}

#[post("/wallet/preSendMoney")]
async fn pre_send_money(
    request: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::pre_send_money::req(request, request_data).await)
}






#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DirectSendMoneyRequest {
    tx_raw: String,
    signature: String,
    device_id: String,
}


#[post("/wallet/directSendMoney")]
async fn direct_send_money(
    request: HttpRequest,
    request_data: web::Json<DirectSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::direct_send_money::req(request, request_data).await)
}



#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReactPreSendMoney {
    tx_id: String,
    is_agreed: bool,
}


#[post("/wallet/reactPreSendMoney")]
async fn react_pre_send_money(
    request: HttpRequest,
    request_data: web::Json<ReactPreSendMoney>,
) -> impl Responder {
    gen_extra_respond(handlers::react_pre_send_money::req(request, request_data).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconfirmSendMoneyRequest {
    device_id: String,
    tx_id: String,
    is_confirmed: bool,
}
#[post("/wallet/reconfirmSendMoney")]
async fn reconfirm_send_money(
    request: HttpRequest,
    request_data: web::Json<ReconfirmSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::reconfirm_send_money::req(request, request_data).await)
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


#[post("/wallet/uploadTxSignature")]
async fn upload_tx_signed_data(
    request: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::upload_tx_signed_data::req(request, request_data).await)
}




/**
 * @api {post} /wallet/backupSecretKeys  私钥密文备份
 * @apiVersion 0.0.1
 * @apiName backupSecretKeys
 * @apiGroup Wallet
 * @apiBody {String} accountId   链上账户ID
 * @apiBody {Object[]} keys  私钥密闻和属性
 * @apiBody {String} [keys[keyData]] 密文
 * @apiBody {String=Master,Servant} [keys[keyType]]  密钥类型
 * @apiBody {String} [keys[deviceId]] 设备ID
 * @apiBody {String} [keys[deviceType]] 手机型号
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8069/wallet/backupSecretKeys
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
 *   OiJIUzI1NiJ9.eyJzdWIiOiJjaGFpbmxlc3MgdXNlcmlkOiAgNCIsImlhdCI6MTcwMzk1Njk5NywiZXhw
 * IjoxNzA1MjUyOTk3fQ.usNdrNVo2oMO0rMdW62rbbooxzOKZjoji9cNN2b1I1c'
 * -d'{"deviceId": "00001","txId": "abc123","signature":"abc123"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8069/wallet/backupSecretKeys
 */

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretKey {
    key_data: String,
    key_type: SecretKeyType,
    device_id:String,
    device_type:String
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecretKeys {
    account_id:String,
    keys: Vec<SecretKey>,
}
/****
#[post("/wallet/backupSecretKeys")]
async fn backup_secret_keys(
    req: HttpRequest,
    request_data: web::Json<uploadTxSignatureRequest>,
){
    unimplemented!()
}

 */
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
        user_info.predecessor = user_id.to_string();
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
        user_info.predecessor = user_id.to_string();
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
