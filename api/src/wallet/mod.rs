//! account manager http service

mod handlers;
mod transaction;

use actix_web::{get, post, web, HttpRequest, Responder};

use serde::{Deserialize, Serialize};

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
async fn search_message_by_account_id(
    request: HttpRequest,
    query_params: web::Query<searchMessageByAccountIdRequest>,
) -> impl Responder {
    gen_extra_respond(
        handlers::search_message::req_by_account_id(request, query_params.into_inner()).await,
    )
}

#[get("/wallet/getStrategy")]
async fn get_strategy(
    request: HttpRequest,
    query_params: web::Query<searchMessageByAccountIdRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::get_strategy::req(request, query_params.into_inner()).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreSendMoneyRequest {
    device_id: String,
    from: String,
    to: String,
    coin: String,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
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
    tx_index: u32,
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
    tx_index: u32,
    confirmed_sig: Option<String>,
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
    tx_index: u32,
    signature: String,
}

#[post("/wallet/uploadServantSig")]
async fn upload_servant_sig(
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
    account_id: String,
    device_id: String,
    pubkey: String,
    secret_key_data: String,
}

#[post("/wallet/addServant")]
async fn add_servant(
    req: HttpRequest,
    request_data: web::Json<AddServantRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::add_servant::req(req, request_data.0).await)
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigRankExternal {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategy {
    account_id: String,
    device_id: String,
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
    use super::*;
    use std::default::Default;

    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;

    use actix_web::{body::MessageBody as _, test, App};

    use models::coin_transfer::CoinTxView;
    use models::secret_store;
    use serde_json::json;

    use blockchain::multi_sig::MultiSig;
    use blockchain::multi_sig::StrategyData;
    use common::data_structures::secret_store::SecretStore;
    use common::http::BackendRespond;

    struct TestWallet {
        account_id: String,
        pubkey: String,
        prikey: String,
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
        models::general::table_all_clear();
        App::new()
            .service(search_message)
            .service(pre_send_money)
            .service(direct_send_money)
            .service(react_pre_send_money)
            .service(reconfirm_send_money)
            .service(upload_servant_sig)
            .service(add_servant)
            .service(update_strategy)
            .service(search_message_by_account_id)
            .service(get_strategy)
    }

    fn simulate_sender_master() -> TestWulianApp {
        let wallet = TestWallet{
            account_id: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
            pubkey: "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0".to_string(),
            prikey: "ed25519:3rjQYUzeLzuX3M7ZAapNjjWnbRjmZrJqarVJrS8vHbhQrqMhLFpWDJafrkSnqdYbbBarzWoB5rb9QXWVDucffyy1".to_string(),
        };

        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjo\
        xLCJkZXZpY2VfaWQiOiIxIiwiaWF0IjoxNzA2ODQ1ODE3NjExLCJleHAiOjE3MDgxNDE4MTc2M\
        TF9.gcfCRP9gND0NaWfswEW5wI34xzaVfkHlZtuSx2VYfEA"
            .to_string();
        let app = TestWulianApp {
            user_id: 1u32,
            token,
            password: "123456789".to_string(),
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
            account_id: app.wallets.first().unwrap().account_id.clone(),
            user_id: app.user_id,
            master_encrypted_prikey: app.wallets.first().unwrap().prikey.clone(),
            servant_encrypted_prikeys: vec![],
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
            password: "123456789".to_string(),
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
            password: "123456789".to_string(),
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
            account_id: app.wallets.first().unwrap().account_id.clone(),
            user_id: app.user_id,
            master_encrypted_prikey: app.wallets.first().unwrap().prikey.clone(),
            servant_encrypted_prikeys: vec![],
        };
        secret_store::single_insert(&data).unwrap();
        app
    }

    async fn clear_contract(account_id: &str) {
        let cli = blockchain::ContractClient::<MultiSig>::new();
        cli.init_strategy(account_id, account_id.to_owned())
            .await
            .unwrap();
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
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/addServant",
            Some(payload),
            Some(&sender_master.token)
        );
        println!("{:?}", res.data);
        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

        //step1: sender master update strategy
        let payload = r#"
            {
             "accountId": "2fa7ab5bd3a75f276fd551aff10b215cf7c8b869ad245b562c55e49f322514c0",
             "deviceId": "1",
             "strategy": [{"min": 0, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200, "sigNum": 1}]
            }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/updateStrategy",
            Some(payload),
            Some(&sender_master.token)
        );
        println!("{:?}", res.data);
        //给链上确认一些时间
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

        //step1.1: check sender's new strategy
        let url = format!(
            "/wallet/getStrategy?accountId={}",
            sender_master.wallets.first().unwrap().account_id
        );
        let res: BackendRespond<StrategyData> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_master.token)
        );
        let sender_strategy = res.data;
        println!("{:?}", sender_strategy);

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
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/preSendMoney",
            Some(payload),
            Some(&sender_master.token)
        );
        println!("{:?}", res.data);

        //step3(optional): check new message
        //for sender master
        let url = format!(
            "/wallet/searchMessageByAccountId?accountId={}",
            sender_master.wallets.first().unwrap().account_id
        );
        println!("{}", url);
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_master.token)
        );
        let tx = res.data.first().unwrap();
        //对于created状态的交易来说，主设备不处理，从设备上传签名
        assert_eq!(tx.transaction.status, CoinTxStatus::Created);
        assert_eq!(
            sender_master.wallets.first().unwrap().pubkey,
            sender_strategy.main_device_pubkey,
            "this device hold  master key,and do nothing for 'Created' tx"
        );

        //step3.1: sender servant sign
        let url = format!(
            "/wallet/searchMessageByAccountId?accountId={}",
            sender_servant.wallets.first().unwrap().account_id
        );
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_servant.token)
        );
        let tx = res.data.first().unwrap();
        //对于created状态的交易来说，主设备不处理，从设备上传签名,其他设备不进行通知
        assert_eq!(tx.transaction.status, CoinTxStatus::Created);
        assert_eq!(
            sender_servant.wallets.first().unwrap().pubkey,
            *sender_strategy.servant_device_pubkey.first().unwrap(),
            "this device hold  servant key,need to sig tx"
        );
        //local sign
        let signature = blockchain::multi_sig::sign_data_by_near_wallet2(
            &sender_servant.wallets.first().unwrap().prikey,
            &tx.transaction.coin_tx_raw,
        );
        //pubkey + signature
        let signature = format!(
            "{}{}",
            sender_servant.wallets.first().unwrap().pubkey,
            signature
        );
        println!("signature {}", signature);
        //upload_servant_sig
        let payload = json!({
        "deviceId":  "2".to_string(),
        "txIndex": tx.tx_index,
        "signature": signature,
        });
        let payload = payload.to_string();
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/uploadServantSig",
            Some(payload),
            Some(&sender_servant.token)
        );
        println!("{:?}", res.data);

        //接受者仅关注SenderSigCompleted状态的交易
        let url = format!(
            "/wallet/searchMessageByAccountId?accountId={}",
            receiver.wallets.first().unwrap().account_id
        );
        let res: BackendRespond<Vec<CoinTxView>> =
            test_service_call!(service, "get", &url, None::<String>, Some(&receiver.token));
        let tx = res.data.first().unwrap();
        assert_eq!(tx.transaction.status, CoinTxStatus::SenderSigCompleted);
        let payload = json!({
        "txIndex": tx.tx_index,
        "isAgreed": true,
        });
        let payload = payload.to_string();
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/reactPreSendMoney",
            Some(payload),
            Some(&receiver.token)
        );
        println!("{:?}", res.data);

        //step5: reconfirm_send_money
        let url = format!(
            "/wallet/searchMessageByAccountId?accountId={}",
            sender_master.wallets.first().unwrap().account_id
        );
        println!("{}", url);
        let res: BackendRespond<Vec<CoinTxView>> = test_service_call!(
            service,
            "get",
            &url,
            None::<String>,
            Some(&sender_master.token)
        );
        let tx = res.data.first().unwrap();
        //对于ReceiverApproved状态的交易来说，只有主设备有权限处理
        //todo: 为了减少一个接口以及减掉客户端交易组装的逻辑，在to账户确认的时候就生成了txid和raw_data,所以master只有1分钟的确认时间
        //超过了就链上过期（非多签业务过期）
        assert_eq!(tx.transaction.status, CoinTxStatus::ReceiverApproved);
        assert_eq!(
            sender_master.wallets.first().unwrap().pubkey,
            sender_strategy.main_device_pubkey,
            "this device hold  master key,and do nothing for 'ReceiverApproved' tx"
        );
        //master_from local sig

        let signature = blockchain::multi_sig::sign_data_by_near_wallet2(
            &sender_master.wallets.first().unwrap().prikey,
            tx.transaction.tx_id.as_ref().unwrap(),
        );

        let payload = json!({
        "deviceId": "1".to_string(),
            "txIndex": tx.tx_index,
        "confirmedSig": signature
        });
        let payload = payload.to_string();
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/wallet/reconfirmSendMoney",
            Some(payload),
            Some(&receiver.token)
        );
        println!("{:?}", res.data);
    }
}
