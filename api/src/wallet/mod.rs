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
use blockchain::multi_sig::{CoinTx, MultiSigRank};
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

#[get("/wallet/searchMessage")]
async fn search_message(request: HttpRequest) -> impl Responder {
    gen_extra_respond(handlers::search_message::req(request).await)
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct searchMessageByAccountIdRequest {
    account_id: String,
}
#[get("/wallet/searchMessageByAccountId")]
async fn search_message_by_account_id(request: HttpRequest,
                                      query_params: web::Query<searchMessageByAccountIdRequest>
) -> impl Responder {
    gen_extra_respond(handlers::search_message::req_by_account_id(request,query_params.into_inner()).await)
}

#[get("/wallet/getStrategy")]
async fn get_strategy(request: HttpRequest,
                                      query_params: web::Query<searchMessageByAccountIdRequest>
) -> impl Responder {
    gen_extra_respond(handlers::get_strategy::req(request,query_params.into_inner()).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    device_id: String,
    from: String,
    to: String,
    coin:String,
    amount:u128,
    expire_at: u64,
    memo:Option<String>
}

#[post("/wallet/preSendMoney")]
async fn pre_send_money(
    request: HttpRequest,
    request_data: web::Json<PreSendMoneyRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::pre_send_money::req(request, request_data.0).await)
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
    gen_extra_respond(handlers::upload_servant_sig::req(request, request_data).await)
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
pub struct AddServantRequest {
    account_id:String,
    device_id:String,
    pubkey:String,
    secret_key_data: String,
}

#[post("/wallet/addServant")]
async fn add_servant(
    req: HttpRequest,
    request_data: web::Json<AddServantRequest>,
) -> impl Responder{
    gen_extra_respond(handlers::add_servant::req(req, request_data.0).await)
}


#[derive(Deserialize, Serialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigRankExternal {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategy {
    account_id:String,
    device_id:String,
    strategy: Vec<MultiSigRankExternal>,
}

#[post("/wallet/updateStrategy")]
async fn update_strategy(
    req: HttpRequest,
    request_data: web::Json<UpdateStrategy>,
) -> impl Responder {
    gen_extra_respond(handlers::update_strategy::req(req, request_data).await)
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use super::*;

    use actix_http::Request;
    use actix_web::body::MessageBody;
    use actix_web::dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::middleware::ErrorHandlerResponse::Response;
    use actix_web::{body, body::MessageBody as _, rt::pin, test, web, App};
    use common::data_structures::wallet::Wallet;
    use models::coin_transfer::CoinTxView;
    use models::{secret_store, wallet};
    use serde_json::json;
    use blockchain::ContractClient;
    use blockchain::multi_sig::MultiSig;
    use common::data_structures::secret_store::SecretStore;
    use common::http::BackendRespond;
    use blockchain::multi_sig::StrategyData;


    struct TestWallet{
        account_id:String,
        pubkey:String,
        prikey:String
    }
    struct TestWulianApp{
        user_id:u32,
        token:String,
        contact:String,
        password:String,
        wallets: Vec<TestWallet>
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
        models::general::table_all_clear();
        App::new()
            .service(search_message)
            .service(pre_send_money)
            .service(direct_send_money)
            .service(react_pre_send_money)
            .service(reconfirm_send_money)
            .service(upload_tx_signed_data)
            .service(add_servant)
            .service(update_strategy)
            .service(search_message_by_account_id)
            .service(get_strategy)

    }

    fn simulate_sender_master() -> TestWulianApp {
        let wallet = TestWallet{
            account_id: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
            pubkey: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
            prikey: "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".to_string(),
        };

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjo\
        xLCJkZXZpY2VfaWQiOiIxIiwiaWF0IjoxNzA2ODQ1ODE3NjExLCJleHAiOjE3MDgxNDE4MTc2M\
        TF9.gcfCRP9gND0NaWfswEW5wI34xzaVfkHlZtuSx2VYfEA"
            .to_string();
        let app = TestWulianApp {
            user_id: 1u32,
            token,
            password:"123456789".to_string(),
            contact: "test@gmail.com".to_string(),
            wallets: vec![wallet],
        };

        let mut user_info = UserInfo::default();
        user_info.email = app.contact.clone();
        user_info.pwd_hash = app.password.clone();
        user_info.account_ids = vec![app.wallets.first().unwrap().account_id.clone()];
        user_info.invite_code = app.user_id.to_string();
        account_manager::single_insert(&user_info).unwrap();
        let data = SecretStore {
            account_id:app.wallets.first().unwrap().account_id.clone(),
            user_id:app.user_id,
            master_encrypted_prikey: app.wallets.first().unwrap().prikey.clone(),
            servant_encrypted_prikeys: vec![]
        };
        secret_store::single_insert(&data).unwrap();

        app
    }

    fn simulate_sender_servant() -> TestWulianApp {
        //deviceId 2
        let wallet = TestWallet{
            account_id: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
            pubkey: "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e".to_string(),
            prikey: "ed25519:s1sw1PXCkHrbyE9Rmg6j18PoUxnhCNZ2CxSPUvvE7dZK9UCEkpTWC1Zy6ZKWvBcAdK8MoRUSdMsduMFRJrRtuGq".to_string(),
        };

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJ\
        kZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHAiOjE3MDgxNDE4ODA4Mjd9.YsI4I\
        9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8"
            .to_string();
        let app = TestWulianApp {
            user_id: 1u32,
            token,
            password:"123456789".to_string(),
            contact: "test@gmail.com".to_string(),
            wallets: vec![wallet],
        };

        app
    }

    fn simulate_receiver() -> TestWulianApp {
        let wallet = TestWallet{
            account_id: "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb".to_string(),
            pubkey: "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb".to_string(),
            prikey: "ed25519:MRLfJQQRGVb8R4vYJLerKXUtsJjHnDG7pYV2jjWShy9svNvk8r5yeVpgY2va6ivHkiZwnyuCMbNPMEN5tH9tK6S".to_string(),
        };

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoyL\
        CJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA3MDIwOTA4OTQzLCJleHAiOjE3MDgzMTY5MDg5ND\
        N9.sgSGOuhVDUTWglXZWfIdo6S-1baQ-hrsbUokrBOA_HU"
            .to_string();
        let app = TestWulianApp {
            user_id: 2u32,
            token,
            password:"123456789".to_string(),
            contact: "tes2@gmail.com".to_string(),
            wallets: vec![wallet],
        };

        let mut user_info = UserInfo::default();
        user_info.email = app.contact.clone();
        user_info.pwd_hash = app.password.clone();
        user_info.account_ids = vec![app.wallets.first().unwrap().account_id.clone()];
        user_info.invite_code = app.user_id.to_string();
        account_manager::single_insert(&user_info).unwrap();
        let data = SecretStore {
            account_id:app.wallets.first().unwrap().account_id.clone(),
            user_id:app.user_id,
            master_encrypted_prikey: app.wallets.first().unwrap().prikey.clone(),
            servant_encrypted_prikeys: vec![]
        };
        secret_store::single_insert(&data).unwrap();
        app
    }

    async fn clear_contract(account_id:&str){
        let cli = blockchain::ContractClient::<MultiSig>::new();
        cli.init_strategy(account_id,account_id.to_owned()).await.unwrap();
    }

    #[actix_web::test]
    async fn test_all_braced_wallet_ok() {
        //1.node0 send 10 cly to 2.node0

        let app = init().await;
        let service = test::init_service(app).await;
        let sender_master = simulate_sender_master();
        let receiver = simulate_receiver();
        let sender_servant = simulate_sender_servant();
        clear_contract("2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0").await;


        //step0: sender master add servant
        let payload = r#"
            {
             "accountId": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "deviceId": "1",
             "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e",
             "secretKeyData": "ed25519:s1sw1PXCkHrbyE9Rmg6j18PoUxnhCNZ2CxSPUvvE7dZK9UCEkpTWC1Zy6ZKWvBcAdK8MoRUSdMsduMFRJrRtuGq"
            }"#;
        let res: BackendRespond<String> = test_service_call!(service,"post","/wallet/addServant",Some(payload),Some(&sender_master.token));
        println!("{:?}",res.data);

        //step1: sender master update strategy
        let payload = r#"
            {
             "accountId": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "deviceId": "1",
             "strategy": [{"min": 0, "maxEq": 1000, "sigNum": 0},{"min": 1000, "maxEq": 1844674407370955200, "sigNum": 1}]
            }"#;
        let res: BackendRespond<String> = test_service_call!(service,"post","/wallet/updateStrategy",Some(payload),Some(&sender_master.token));
        println!("{:?}",res.data);
        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;


        //step1.1: check sender's new strategy
        let url = format!("/wallet/getStrategy?accountId={}",sender_master.wallets.first().unwrap().account_id);
        let res : BackendRespond<StrategyData> = test_service_call!(service,"get",&url, None::<String>,Some(&sender_master.token));
        let sender_strategy = res.data;
        println!("{:?}",sender_strategy);


        //step2: master: pre_send_money
        let payload = r#"
            {
             "deviceId": "1",
             "from": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "to": "535ff2aeeb5ea8bcb1acfe896d08ae6d0e67ea81b513f97030230f87541d85fb",
             "coin":"dw20",
             "amount": 123,
             "expireAt": 1708015513000
            }"#;
        let res: BackendRespond<String> = test_service_call!(service,"post","/wallet/preSendMoney",Some(payload),Some(&sender_master.token));
        println!("{:?}",res.data);

        //step3(optional): check new message
        //for sender master
        let url = format!("/wallet/searchMessageByAccountId?accountId={}",sender_master.wallets.first().unwrap().account_id);
        println!("{}",url);
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_master.token)
        );
        let tx = res.data.first().unwrap();
        //对于created状态的交易来说，主设备不处理，从设备上传签名
        if tx.transaction.status == CoinTxStatus::Created {
            if sender_master.wallets.first().unwrap().pubkey == sender_strategy.main_device_pubkey {
                println!("this device hold  master key,and do nothing for 'Created' tx");
            }
        }

        //for sender servant
        let url = format!("/wallet/searchMessageByAccountId?accountId={}", sender_servant.wallets.first().unwrap().account_id);
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_servant.token)
        );
        let tx = res.data.first().unwrap();
        //对于created状态的交易来说，主设备不处理，从设备上传签名,其他设备不进行通知
        if tx.transaction.status == CoinTxStatus::Created {
            //从设备需要去本地签名
            if sender_servant.wallets.first().unwrap().pubkey == *sender_strategy.servant_device_pubkey.first().unwrap() {
                println!("this device hold  servant key,need to sig tx");
                //todo: local sign
            }
        }

        //接受者不关注created状态的交易
        let url = format!("/wallet/searchMessageByAccountId?accountId={}", receiver.wallets.first().unwrap().account_id);
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&receiver.token)
        );
        println!("{:?}",res.data);

        //step3: servent: upload_servant_sig
        //step4: react_pre_send_money
        //step5: reconfirm_send_money



        /***
        let req = test::TestRequest::post()
            .uri("/wallet/addServant")
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

        //step0.1: sender set new strategy



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
*/
        //for receiver after preSendMoney
        /***
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

         */
    }
}
