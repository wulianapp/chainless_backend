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
use blockchain::multi_sig::{StrategyData};
use common::encrypt::{ed25519_key_gen};
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
use crate::wallet::handlers::get_tx::CoinTxViewTmp2;

#[derive(Debug)]
pub struct TestWallet {
    pub main_account: String,
    pub pubkey: Option<String>,
    pub prikey: Option<String>,
    pub subaccount: Vec<String>,
    pub sub_prikey: Option<Vec<String>>,
}
#[derive(Debug)]
pub struct TestDevice {
    pub id: String,
    pub brand: String,
}
#[derive(Debug)]
pub struct TestUser {
    pub contact: String,
    pub password: String,
    pub captcha: String,
    pub token: Option<String>,
}

#[derive(Debug)]
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
    env::set_var("BACKEND_SERVICE_MODE", "test");
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
            captcha: "000000".to_string(),
            token: None,
        },
        device: TestDevice{
            id: "1".to_string(),
            brand: "Apple_Master".to_string(),
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
            captcha: "000000".to_string(),
            token: None,
        },
        device: TestDevice {
            id: "2".to_string(),
            brand: "Apple_Servant".to_string(),
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
            captcha: "000000".to_string(),
            token: None,
        },
        device: TestDevice {
            id: "4".to_string(),
            brand: "Apple_Newcommer".to_string(),
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
            captcha: "000000".to_string(),
            token: None,
        },
        device: TestDevice{
            id: "3".to_string(),
            brand: "Huawei_Receiver".to_string(),
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


pub fn gen_some_accounts_with_new_key() ->(TestWulianApp2,TestWulianApp2,TestWulianApp2,TestWulianApp2){
    let sender_master_secret = ed25519_key_gen();
    let sender_sub_secret = ed25519_key_gen();
    let sender_servant_secret = ed25519_key_gen();
    let sender_newcommer_secret = ed25519_key_gen();
    let receiver_master_secret = ed25519_key_gen();
    let receiver_sub_secret = ed25519_key_gen();
 
    let mut sender_master = simulate_sender_master();
    sender_master.wallet = TestWallet {
        main_account: sender_master_secret.1.clone(),
        pubkey: Some(sender_master_secret.1.clone()),
        prikey: Some(sender_master_secret.0.clone()),
        subaccount: vec![sender_sub_secret.1.clone()],
        sub_prikey: Some(vec![sender_sub_secret.0.clone()]),
    };

    let mut receiver = simulate_receiver();
    receiver.wallet = TestWallet {
        main_account: receiver_master_secret.1.clone(),
        pubkey: Some(receiver_master_secret.1),
        prikey: Some(receiver_master_secret.0),
        subaccount: vec![receiver_sub_secret.1],
        sub_prikey: Some(vec![receiver_sub_secret.0]),
    };

    let mut sender_servant = simulate_sender_servant();
    sender_servant.wallet = TestWallet {
        main_account: sender_master_secret.1.clone(),
        pubkey: Some(sender_servant_secret.1.clone()),
        prikey: Some(sender_servant_secret.0.clone()),
        subaccount: vec![sender_sub_secret.1.clone()],
        sub_prikey: None,
    };

    let mut sender_newcommer = simulate_sender_new_device();
    sender_newcommer.wallet = TestWallet {
        main_account: "".to_string(),
        pubkey: Some(sender_newcommer_secret.1.clone()),
        prikey: Some(sender_newcommer_secret.0.clone()),
        subaccount: vec![],
        sub_prikey: None,
    };
    let all_accounts = (sender_master,sender_servant,sender_newcommer,receiver);
    println!("{:#?}",all_accounts);
    all_accounts
}

pub async fn clear_contract() {
    let cli = blockchain::ContractClient::<MultiSig>::new().unwrap();
    cli.clear_all().await.unwrap();
    //cli.init_strategy(account_id, account_id.to_owned()).await.unwrap();
    //cli.remove_account_strategy(account_id.to_owned()).await.unwrap();
    //cli.remove_tx_index(1u64).await.unwrap();
}

pub async fn get_tx_status_on_chain(txs_index: Vec<u64>) -> Vec<(u64, bool)> {
    let cli = blockchain::ContractClient::<MultiSig>::new().unwrap();
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
//data值都放上次处理进行unwrap，因为确实有null的场景，没有需要数据返回的直接不返回数据
#[macro_export]
macro_rules! test_register {
    ( $service:expr,$app:expr) => {{
            let payload = json!({
                "deviceId":  $app.device.id,
                "contact": $app.user.contact,
                "kind": "Register"
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
                //"captcha": $app.user.captcha,
                "captcha": "000000",
                "password": $app.user.password
            });

            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/registerByEmail",
                Some(payload.to_string()),
                None::<String>
            );
            $app.user.token = Some(res.data.unwrap());
    }};
}


#[macro_export]
macro_rules! test_get_captcha_with_token {
    ( $service:expr,$app:expr,$kind:expr) => {{
            let payload = json!({
                "contact":$app.user.contact,
                "kind": $kind
            });
            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/getCaptchaWithToken",
                Some(payload.to_string()),
                Some($app.user.token.as_ref().unwrap())
            );
            assert_eq!(res.status_code,0);
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
            $app.user.token = Some(res.data.unwrap());
    }};
}

#[macro_export]
macro_rules! test_create_main_account{
    ($service:expr, $app:expr) => {{
        let payload = json!({
            "contact": $app.user.contact,
            "kind": "SetSecurity"
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/accountManager/getCaptchaWithToken",
            Some(payload.to_string()),
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);

        let payload = json!({
            "masterPubkey":  $app.wallet.main_account,
            "masterPrikeyEncryptedByPassword": $app.wallet.prikey,
            "masterPrikeyEncryptedByAnswer": $app.wallet.prikey,
            "subaccountPubkey":  $app.wallet.subaccount.first().unwrap(),
            "subaccountPrikeyEncrypedByPassword": $app.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "subaccountPrikeyEncrypedByAnswer": $app.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "anwserIndexes": "",
            "captcha": "000000"
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
        let url = format!("/wallet/getStrategy");
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
    }};
}

#[macro_export]
macro_rules! test_servant_saved_secret {
    ($service:expr,$servant:expr) => {{
        let url = format!("/wallet/servantSavedSecret");
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            &url,
            None::<String>,
            Some($servant.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
    }};
}

#[macro_export]
macro_rules! test_add_subaccount {
    ($service:expr, $master:expr) => {{
        let payload = json!({
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
    }};
}

//sender main_account update strategy
#[macro_export]
macro_rules! test_update_strategy {
    ($service:expr, $master:expr) => {{
        let payload = json!({
            "deviceId": "1",
            "strategy": [{"min": 1, "maxEq": 10, "sigNum": 0},{"min": 10, "maxEq": 1844674407370955200u64, "sigNum": 1}]
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/updateStrategy",
            Some(payload.to_string()),
            Some($master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
    }};
}



#[macro_export]
macro_rules! test_update_security {
    ($service:expr, $app:expr, $secrets:expr) => {{
        let payload = json!({
            "contact": $app.user.contact,
            "kind": "SetSecurity"
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/accountManager/getCaptchaWithToken",
            Some(payload.to_string()),
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);


        let payload = json!({
            "secrets": $secrets,
            "anwserIndexes": "1,2,3",
            "captcha": "000000"
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/updateSecurity",
            Some(payload.to_string()),
            Some($app.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
    }};
}



#[macro_export]
macro_rules! test_pre_send_money {
    ($service:expr, $sender_master:expr, $receiver_account:expr,$coin:expr,$amount:expr,$is_forced:expr,$captcha:expr) => {{
        let payload = json!({
            "to": &$receiver_account,
            "coin":$coin,
            "amount": $amount,
            "expireAt": 1808015513000u64,
            "isForced": $is_forced,
            "captcha": $captcha,
       });
        let res: BackendRespond<(u32,Option<String>)> = test_service_call!(
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
macro_rules! test_pre_send_money2 {
    ($service:expr, $sender_master:expr, $receiver_account:expr,$coin:expr,$amount:expr,$is_forced:expr) => {{
        let payload = json!({
            "to": &$receiver_account,
            "coin":$coin,
            "amount": $amount,
            "expireAt": 1808015513000u64,
            "isForced": $is_forced,
            //"captcha": $captcha,
       });
        let res: BackendRespond<(u32,Option<String>)> = test_service_call!(
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
macro_rules! test_pre_send_money_to_sub {
    ($service:expr, $sender_master:expr, $receiver_account:expr,$coin:expr,$amount:expr) => {{
        let payload = json!({
            "to": &$receiver_account,
            "coin":$coin,
            "amount": $amount,
            "expireAt": 1808015513000u64,
            "captcha": "000000",
       });
        let res: BackendRespond<(u32,String)> = test_service_call!(
            $service,
            "post",
            "/wallet/preSendMoneyToSub",
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_reconfirm_send_money {
    ($service:expr, $sender_master:expr, $index:expr,$signature:expr) => {{
        let payload = json!({
            "txIndex": $index,
            "confirmedSig": $signature,
       });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/reconfirmSendMoney",
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
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
    }};
}



#[macro_export]
macro_rules! test_newcommer_switch_servant {
    ($service:expr, $sender_master:expr,$sender_servant:expr,$sender_new_device:expr) => {{
        let payload = json!({
            "oldServantPubkey": $sender_servant.wallet.pubkey.unwrap(),
            "newServantPubkey": $sender_new_device.wallet.pubkey.unwrap(),
            "newServantPrikeyEncrypedByPassword": $sender_new_device.wallet.prikey.clone().unwrap(),
            "newServantPrikeyEncrypedByAnswer": $sender_new_device.wallet.prikey.unwrap(),
            "newDeviceId": $sender_new_device.device.id
        });
        let res: BackendRespond<(u32,String)> = test_service_call!(
            $service,
            "post",
            "/wallet/newcommerSwitchServant",
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_gen_newcommer_switch_master {

    ($service:expr, $sender_newcommer:expr) => {{
        let payload = json!({
            "newcomerPubkey":  $sender_newcommer.wallet.pubkey.clone().unwrap(),
            "captcha":"000000"
        });
        let url = format!("/wallet/genNewcomerSwitchMaster");
        let res: BackendRespond<super::handlers::gen_newcomer_switch_master::GenReplaceKeyInfo> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($sender_newcommer.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}



#[macro_export]
macro_rules! test_gen_servant_switch_master {
    ($service:expr,$sender_servant:expr) => {{
        let payload = json!({
            "captcha": "000000",
        });
        let url = format!("/wallet/genServantSwitchMaster");
        let res: BackendRespond<super::handlers::gen_newcomer_switch_master::GenReplaceKeyInfo> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($sender_servant.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_sub_send_to_master {
    ($service:expr,$sender_master:expr,$signature:expr,$coin:expr,$amount:expr) => {{
        let payload = json!({
            "subSig": $signature,
            "coin":   $coin,
            "amount": $amount
        });
        let url = format!("/wallet/subSendToMain");
        let res: BackendRespond<super::handlers::gen_newcomer_switch_master::GenReplaceKeyInfo> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_update_subaccount_hold_limit {
    ($service:expr,$sender_master:expr,$subaccount:expr,$limit:expr) => {{
        let payload = json!({
            "subaccount": $subaccount,
            "limit": $limit
        });
        let url = format!("/wallet/updateSubaccountHoldLimit");
        let res: BackendRespond<super::handlers::gen_newcomer_switch_master::GenReplaceKeyInfo> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($sender_master.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_react_pre_send_money {
    ($service:expr,$receiver:expr,$index:expr,$is_agreed:expr) => {{
        let payload = json!({
            "txIndex": $index,
            "isAgreed": $is_agreed,
        });
        let url = format!("/wallet/reactPreSendMoney");
        let res: BackendRespond<super::handlers::gen_newcomer_switch_master::GenReplaceKeyInfo> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($receiver.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}

#[macro_export]
macro_rules! test_commit_newcommer_switch_master {
    ($service:expr, $sender_newcommer:expr,$gen_res:expr,$add_key_sig:expr,$delete_key_sig:expr) => {{
        let payload = json!({
            "newcomerPubkey":  $sender_newcommer.wallet.pubkey.unwrap(),
            "addKeyRaw":  $gen_res.as_ref().unwrap().add_key_raw,
            "deleteKeyRaw":  $gen_res.as_ref().unwrap().delete_key_raw,
            "addKeySig":  $add_key_sig,
            "deleteKeySig": $delete_key_sig,
            "newcomerPrikeyEncryptedByPassword":  "".to_string(),
            "newcomerPrikeyEncryptedByAnswer":  "".to_string()
        });

        println!("{:?}", payload.to_string());
        let url = format!("/wallet/commitNewcomerSwitchMaster");
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            &url,
            Some(payload.to_string()),
            Some($sender_newcommer.user.token.as_ref().unwrap())
        );
        assert_eq!(res.status_code,0);
        res.data
    }};
}


#[macro_export]
macro_rules! test_commit_servant_switch_master {
    ($service:expr, $sender_servant:expr,$gen_res:expr,$add_key_sig:expr,$delete_key_sig:expr) => {{
        let payload = json!({
            "addKeyRaw":  $gen_res.as_ref().unwrap().add_key_raw,
            "deleteKeyRaw":  $gen_res.as_ref().unwrap().delete_key_raw,
            "addKeySig":  $add_key_sig,
            "deleteKeySig": $delete_key_sig,
        });

        //claim
        println!("{:?}", payload.to_string());
        let url = format!("/wallet/commitServantSwitchMaster");
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            &url,
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
macro_rules! test_get_tx {
    ($service:expr, $app:expr,$index:expr) => {{
        let url = format!("/wallet/getTx?index={}",$index);
        let res: BackendRespond<CoinTxViewTmp2> = test_service_call!(
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



