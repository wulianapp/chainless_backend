use crate::account_manager::{configure_routes, handlers};
use crate::utils::respond::BackendRespond;
use crate::{
    test_add_servant, test_create_main_account, test_get_balance_list, test_get_secret,
    test_get_strategy, test_login, test_register, test_search_message, test_service_call,
    test_update_security,
};

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
use common::data_structures::wallet::CoinType;
use models::account_manager::UserInfoView;
use tracing::{debug, error, info};
use crate::wallet::handlers::balance_list::AccountBalance;

pub struct TestWallet {
    pub main_account: String,
    pub pubkey: Option<String>,
    pub prikey: Option<String>,
    pub subaccount: Vec<String>,
    pub sub_prikey: Option<Vec<String>>,
}

pub struct TestDevice {
    pub id: String,
    pub brand: String,
}

pub struct TestUser {
    pub contact: String,
    pub password: String,
    pub captcha: String,
    pub token: Option<String>,
}

pub struct TestWulianApp2 {
    pub user: TestUser,
    pub device: TestDevice,
    pub wallet: TestWallet,
}

pub async fn init() -> App<
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
        .configure(crate::wallet::configure_routes)
}

pub fn simulate_sender_master() -> TestWulianApp2 {
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

pub fn simulate_sender_servant() -> TestWulianApp2 {
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

pub fn simulate_sender_new_device() -> TestWulianApp2 {
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

pub fn simulate_receiver() -> TestWulianApp2 {
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

pub async fn clear_contract() {
    let cli = blockchain::ContractClient::<MultiSig>::new();
    cli.clear_all().await.unwrap();
    //cli.init_strategy(account_id, account_id.to_owned()).await.unwrap();
    //cli.remove_account_strategy(account_id.to_owned()).await.unwrap();
    //cli.remove_tx_index(1u64).await.unwrap();
}

pub async fn get_tx_status_on_chain(txs_index: Vec<u64>) -> Vec<(u64, bool)> {
    let cli = blockchain::ContractClient::<MultiSig>::new();
    cli.get_tx_state(txs_index).await.unwrap().unwrap()
}

#[macro_export]
macro_rules! test_service_call {
    ( $service:expr,$method:expr,$api:expr,$payload:expr,$token:expr) => {{
        let mut parameters = if $method == "post" {
            test::TestRequest::post()
                .uri($api)
                .insert_header(header::ContentType::json())
        } else {
            test::TestRequest::get().uri($api)
        };

        if let Some(data) = $payload {
            parameters = parameters.set_payload(data);
        };

        if let Some(data) = $token {
            parameters =
                parameters.insert_header((header::AUTHORIZATION, format!("bearer {}", data)));
        };

        let req = parameters.to_request();
        let body = test::call_and_read_body(&$service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("body_str {}", body_str);
        serde_json::from_str::<_>(&body_str).unwrap()
    }};
}

#[macro_export]
macro_rules! test_register {
    ( $service:expr,$app:expr) => {{
            let payload = json!({
                "deviceId":  $app.device.id,
                "contact": $app.user.contact,
                "kind": "register"
            });
            let _res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/getCaptcha",
                Some(payload.to_string()),
                None::<String>
            );

            let payload = json!({
                "deviceId":  $app.device.id,
                "deviceBrand": $app.device.brand,
                "email": $app.user.contact,
                "captcha": $app.user.captcha,
                "password": $app.user.password
            });

            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/registerByEmail",
                Some(payload.to_string()),
                None::<String>
            );
            $app.user.token = Some(res.data);
    }};
}

#[macro_export]
macro_rules! test_login {
    ($service:expr, $app:expr) => {{
            let payload = json!({
                "deviceId":  $app.device.id,
                "deviceBrand": $app.device.brand,
                "contact": $app.user.contact,
                "password": $app.user.password
            });
            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/login",
                Some(payload.to_string()),
                None::<String>
            );
            $app.user.token = Some(res.data);
    }};
}

#[macro_export]
macro_rules! test_create_main_account{
    ($service:expr, $app:expr) => {{
        let payload = json!({
            "masterPubkey":  $app.wallet.main_account,
            "masterPrikeyEncryptedByPassword": $app.wallet.prikey,
            "masterPrikeyEncryptedByAnswer": $app.wallet.prikey,
            "subaccountPubkey":  $app.wallet.subaccount.first().unwrap(),
            "subaccountPrikeyEncrypedByPassword": $app.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "subaccountPrikeyEncrypedByAnswer": $app.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "anwserIndexes": ""
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/createMainAccount",
            Some(payload.to_string()),
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
    }};
}

#[macro_export]
macro_rules! test_search_message {
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/searchMessage");
        let res: BackendRespond<Vec<AccountMessage>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_get_strategy {
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/getStrategy?accountId={}", $app.wallet.main_account);
        let res: BackendRespond<StrategyDataTmp> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_tx_list {
    ($service:expr, $app:expr,$role:expr,$counterparty:expr,$per_page:expr,$page:expr) => {{
        let url = match $counterparty {
            Some(acc) => {
                format!("/wallet/txList?txRole={}&counterparty={}&perPage={}&page={}",
                $role.to_string(),acc,$per_page,$page)
            },
            None =>{
                format!("/wallet/txList?txRole={}&perPage={}&page={}",
                $role,$per_page,$page)            
            }
        };
        let res: BackendRespond<Vec<crate::wallet::handlers::tx_list::CoinTxViewTmp>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

//generate servent_key in device which hold master_prikey,and send to server after encrypted
#[macro_export]
macro_rules! test_add_servant {
    ($service:expr, $master:expr, $servant:expr) => {{
        let payload = json!({
            "mainAccount":  $master.wallet.main_account,
            "servantPubkey":  $servant.wallet.pubkey.as_ref().unwrap(),
            "servantPrikeyEncrypedByPassword":  $servant.wallet.prikey.as_ref().unwrap(),
            "servantPrikeyEncrypedByAnswer":  $servant.wallet.prikey.as_ref().unwrap(),
            "holderDeviceId":  $servant.device.id,
            "holderDeviceBrand": $servant.device.brand,
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/addServant",
            Some(payload.to_string()),
            Some($master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_add_subaccount {
    ($service:expr, $master:expr) => {{
        let payload = json!({
            "mainAccount":  $master.wallet.main_account,
            "subaccountPubkey":  $master.wallet.subaccount.first().unwrap(),
            "subaccountPrikeyEncrypedByPassword": "by_password_ead4cf1",
            "subaccountPrikeyEncrypedByAnswer": "byanswer_ead4cf1e",
            "holdValueLimit": 10000,
        });
        let url = format!("/wallet/addSubaccount");
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        res.data
    }};
}



#[macro_export]
macro_rules! test_remove_servant {
    ($service:expr, $master:expr, $servant:expr) => {{
        let payload = json!({
            "servantPubkey":  $sender.wallet.pubkey.as_ref().unwrap(),
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/removeServant",
            Some(payload.to_string()),
            Some($master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

//sender main_account update strategy
#[macro_export]
macro_rules! test_update_strategy {
    ($service:expr, $master:expr) => {{
        let payload = json!({
            "accountId":  $master.wallet.main_account,
            "deviceId": "1",
            "strategy": [{"min": 1, "maxEq": 100, "sigNum": 0},{"min": 100, "maxEq": 1844674407370955200u64, "sigNum": 1}]
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/updateStrategy",
            Some(payload.to_string()),
            Some($master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}



#[macro_export]
macro_rules! test_update_security {
    ($service:expr, $app:expr, $secrets:expr) => {{
        let payload = json!({
            "secrets": $secrets,
            "anwserIndexes": "1,2,3"
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/updateSecurity",
            Some(payload.to_string()),
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}



#[macro_export]
macro_rules! test_pre_send_money {
    ($service:expr, $sender_master:expr, $receiver_account:expr,$coin:expr,$amount:expr) => {{
        let payload = json!({
            "from": &$sender_master.wallet.main_account,
            "to": &$receiver_account,
            "coin":$coin,
            "amount": $amount,
            "expireAt": 1808015513000u64
       });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/preSendMoney",
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_upload_servant_sig {
    ($service:expr, $sender_servant:expr,$tx_index:expr,$signature:expr) => {{
        let payload = json!({
            "txIndex": $tx_index,
            "signature": $signature,
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/uploadServantSig",
            Some(payload.to_string()),
            Some($sender_servant.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}




#[macro_export]
macro_rules! test_faucet_claim {
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/faucetClaim");
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}



//query
#[macro_export]
macro_rules! test_get_secret {
    ($service:expr, $app:expr,$type:expr) => {{
        let url = format!("/wallet/getSecret?type={}", $type);
        let res: BackendRespond<Vec<SecretStore>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_get_balance_list {
    ($service:expr, $app:expr,$kind:expr) => {{
        let url = format!("/wallet/balanceList?kind={}",$kind);
        let res: BackendRespond<Vec<(String,Vec<crate::wallet::handlers::balance_list::AccountBalance>)>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_get_device_list {
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/deviceList");
        let res: BackendRespond<Vec<DeviceInfo>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code, 0);
        res.data
    }};
}



