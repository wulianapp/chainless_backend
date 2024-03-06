#![feature(async_closure)]

//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, Responder};

use serde::{Deserialize, Serialize};

//use captcha::{ContactType, VerificationCode};

use crate::utils::respond::gen_extra_respond;

/**
 * @api {post} /accountManager/getCaptcha 获取验证码
 * @apiVersion 0.0.1
 * @apiName getCaptcha
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   用户设备ID,也是测试服务的验证码返回值
 * @apiBody {String} contact 用户联系方式 手机 +86 18888888888 or 邮箱 test1@gmail.com
 * @apiBody {String="register","resetPassword","setSecurity","addServant","servantReplaceMaster","newcomerBecomeMaster"} kind 验证码类型
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8065/accountManager/getCaptcha -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test1@gmail.com","kind":"register"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/getCaptcha
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptchaRequest {
    device_id: String,
    contact: String,
    kind: String,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/getCaptcha")]
async fn get_captcha(request_data: web::Json<GetCaptchaRequest>) -> impl Responder {
    gen_extra_respond(handlers::get_captcha::req(request_data.into_inner()).await)
}

/**
 * @api {get} /accountManager/contactIsUsed 检查账户是否已被使用
 * @apiVersion 0.0.1
 * @apiName contactIsUsed
 * @apiGroup AccountManager
 * @apiBody {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8065/accountManager/contactIsUsed?contact=test1@gmail.com"
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/contactIsUsed
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/accountManager/contactIsUsed")]
async fn contact_is_used(
    //request_data: web::Json<ContactIsUsedRequest>,1
    query_params: web::Query<ContactIsUsedRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::contact_is_used::req(query_params.into_inner()))
}



/**
 * @api {get} /accountManager/userInfo 用户账号详情
 * @apiVersion 0.0.1
 * @apiName userInfo
 * @apiGroup AccountManager
 * @apiBody {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8065/accountManager/userInfo?contact=test1@gmail.com"
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/userInfo
 */
type UserInfoRequest = ContactIsUsedRequest;
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/accountManager/userInfo")]
async fn user_info(
    //request_data: web::Json<ContactIsUsedRequest>,1
    query_params: web::Query<UserInfoRequest>,
) -> impl Responder {
    gen_extra_respond(handlers::user_info::req(query_params.into_inner()))
}

/**
 * --api {post} /accountManager/verifyCaptcha check verificationCode
 * --apiVersion 0.0.1
 * --apiName verify_captcha
 * --apiGroup AccountManager
 * --apiBody {String} deviceId   user's device id
 * --apiBody {String} userContact example phone +86 18888888888 or email test1@gmail.com
 * --apiBody {String} verificationCode   user's verification code for register
 * --apiBody {String=101,102,103,104} kind   101 register，102 login，103 reset password，104 shortcut login
 * --apiExample {curl} Example usage:
 * curl -X POST http://120.232.251.101:8065/accountManager/getCaptcha -H "Content-Type: application/json" -d
 *   '{"deviceId": "John Doe","userContact": "johne@example.com","kind":"101","verificationCode":"685886"}'
 * --apiSuccess {string} status_code         status code.
 * --apiSuccess {string} msg                 description of status.
 * --apiSuccess {string} data                nothing.
 * --apiSampleRequest http://120.232.251.101:8065/accountManager/verifyCaptcha
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifyCodeRequest {
    device_id: String,
    user_contact: String,
    kind: String,
    captcha: String,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/verifyCaptcha1")]
async fn verify_captcha(request_data: web::Json<VerifyCodeRequest>) -> impl Responder {
    gen_extra_respond(handlers::verify_captcha::req(request_data.into_inner()).await)
}


/**
__apiBody {String} encryptedPrikey    私钥两次私钥加密后密文的拼接
———apiBody {String} pubkey     公钥的hex表达

* @api {post} /accountManager/registerByEmail 通过邮箱注册账户
* @apiVersion 0.0.1
* @apiName registerByEmail
* @apiGroup AccountManager
* @apiBody {String} deviceId  设备ID
* @apiBody {String} deviceBrand  手机型号 Huawei-P20
* @apiBody {String} email     邮箱 test1@gmail.com
* @apiBody {String} captcha   验证码
* @apiBody {String} password     登录密码
* @apiBody {String} [predecessorInviteCode]   推荐人的邀请码
* @apiExample {curl} Example usage:
    curl -X POST http://120.232.251.101:8065/accountManager/registerByEmail -H "Content-Type: application/json" -d
  '{"deviceId": "123","email": "test1@gmail.com","captcha":"000000","password":"123456789","encryptedPrikey": "123",
   "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e"}'
* @apiSuccess {string} status_code         status code.
* @apiSuccess {string} msg                 description of status.
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8065/accountManager/registerByEmail
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterByEmailRequest {
    device_id: String,
    device_brand: String,
    email: String,
    captcha: String,
    password: String,
    //encrypted_prikey: String,
    //pubkey: String,
    predecessor_invite_code: Option<String>,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/registerByEmail")]
async fn register_by_email(request_data: web::Json<RegisterByEmailRequest>) -> impl Responder {
    gen_extra_respond(handlers::register::by_email::req(request_data.into_inner()).await)
}

/**
 * * ——apiBody {String} pubkey     公钥的hex表达
 *  __apiBody {String} encryptedPrikey     私钥两次私钥加密后密文的拼接
* @api {post} /accountManager/registerByPhone 通过手机注册账户
* @apiVersion 0.0.1
* @apiName registerByPhone
* @apiGroup AccountManager
* @apiBody {String} deviceId  设备ID
* @apiBody {String} deviceBrand  手机型号 Huawei-P20
* @apiBody {String} phoneNumber     手机号 +86 18888888888
* @apiBody {String} captcha   验证码
* @apiBody {String} password       密码
* @apiBody {String} [predecessorInviteCode]   推荐人的邀请码
* @apiExample {curl} Example usage:
*    curl -X POST http://120.232.251.101:8065/accountManager/registerByPhone -H "Content-Type: application/json" -d
  '{"deviceId": "123","phoneNumber": "+86 13682000011","captcha":"000000","password":"123456789","encryptedPrikey": "123",
   "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee33","predecessorInviteCode":"1"}'
* @apiSuccess {string} status_code         status code.
* @apiSuccess {string} msg                 description of status.
* @apiSuccess {string} data                jwt token.
* @apiSampleRequest http://120.232.251.101:8065/accountManager/registerByEmail
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterByPhoneRequest {
    device_id: String,
    device_brand: String,
    phone_number: String,
    captcha: String,
    password: String,
    //encrypted_prikey: String,
    //pubkey: String,
    predecessor_invite_code: Option<String>,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/registerByPhone")]
async fn register_by_phone(request_data: web::Json<RegisterByPhoneRequest>) -> impl Responder {
    gen_extra_respond(handlers::register::by_phone::req(request_data.into_inner()).await)
}

/**
 * @api {post} /accountManager/login 登录
 * @apiVersion 0.0.1
 * @apiName login
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   设备ID
 * @apiBody {String} deviceBrand  手机型号 Huawei-P20
 * @apiBody {String} contact    例如 phone +86 18888888888 or email test1@gmail.com
 * @apiBody {String} password   密码 1234
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8065/accountManager/login -H "Content-Type: application/json" -d
 *  '{"deviceId": "1234","contact": "test1@gmail.com","password":"123456789"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                jwt token.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/login
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    device_id: String,
    device_brand: String,
    contact: String,
    password: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/login")]
async fn login(request_data: web::Json<LoginRequest>) -> impl Responder {
    gen_extra_respond(handlers::login::req(request_data.into_inner()).await)
}

/**
* @api {post} /accountManager/resetPassword  重置登录密码
* @apiVersion 0.0.1
* @apiName ResetPassword
* @apiGroup AccountManager
* @apiBody {String} deviceId     设备ID
* @apiBody {String} contact      手机或邮箱 +86 18888888888 or email test1@gmail.com
* @apiBody {String} newPassword  新密码  "abcd"
* @apiBody {String} captcha      验证码  "123456"
* @apiExample {curl} Example usage:
  curl -X POST http://120.232.251.101:8065/accountManager/resetPassword -H "Content-Type: application/json"
 -d '{"deviceId": "123","contact": "test1@gmail.com","captcha":"287695","newPassword":"123456788"}'
* @apiSuccess {string} status_code         status code.
* @apiSuccess {string} msg                 description of status.
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8065/accountManager/resetPassword
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    device_id: String,
    contact: String,
    captcha: String,
    new_password: String,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/resetPassword")]
async fn reset_password(request_data: web::Json<ResetPasswordRequest>) -> impl Responder {
    gen_extra_respond(handlers::reset_password::req(request_data).await)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_captcha)
        .service(contact_is_used)
        .service(verify_captcha)
        .service(register_by_email)
        .service(register_by_phone)
        .service(login)
        .service(user_info)
        .service(reset_password);
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::{test, App, Error};
    use std::env;
    use crate::test_service_call;

    use crate::utils::respond::BackendRespond;
    async fn clear_contract(account_id: &str) {
        let cli = blockchain::ContractClient::<blockchain::multi_sig::MultiSig>::new();
        cli.clear_all().await.unwrap();
        //cli.init_strategy(account_id, account_id.to_owned()).await.unwrap();
        //cli.remove_account_strategy(account_id.to_owned()).await.unwrap();
        //cli.remove_tx_index(1u64).await.unwrap();
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
        App::new().configure(configure_routes)
    }

    #[actix_web::test]
    async fn test_hello() {
        let app = init().await;
        let service = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/hello/test").to_request();
        let body = test::call_service(&service, req)
            .await
            .into_body()
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("{}", body_str);
    }

    #[actix_web::test]
    async fn test_all_braced_account_manager_ok() {
        let app = init().await;
        let service = test::init_service(app).await;
        clear_contract("").await;

        //getCaptcha
        let payload = r#"{ "deviceId": "1", "contact": "test1@gmail.com","kind": "register" }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/getCaptcha",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);

        //register
        let payload = r#"
            { 
            "deviceId": "1",
            "deviceBrand": "Apple",
            "email": "test1@gmail.com",
            "captcha": "000000",
            "password": "123456789"
            }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/registerByEmail",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);

        //login
        let payload = r#"
            { "deviceId": "1",
             "deviceBrand": "Apple",
            "contact": "test1@gmail.com",
             "password": "123456789"
            }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/login",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);

        //check contact if is used
        let res: BackendRespond<bool> = test_service_call!(
            service,
            "get",
            "/accountManager/contactIsUsed?contact=test1@gmail.com",
            None::<String>,
            None::<String>
        );
        println!("{:?}", res.data);

        let payload = r#"{ "deviceId": "1", "contact": "test1@gmail.com","kind": "resetPassword" }"#;
        let res:BackendRespond<String> = test_service_call!(service,"post","/accountManager/getCaptcha",Some(payload),None::<String>);
        println!("{:?}",res.data);
        let payload = r#"
        { "deviceId": "1",
         "captcha": "000000",
         "contact": "test1@gmail.com",
         "newPassword": "new123456789"
        }
        "#;
        let res: BackendRespond<String> = test_service_call!(service,"post","/accountManager/resetPassword",Some(payload),None::<String>);
        println!("{:?}",res.msg);


        
        let payload = r#"{ "deviceId": "2", "contact": "test2@gmail.com","kind": "register" }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/getCaptcha",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);

        //register
        let payload = r#"
            { 
            "deviceId": "2",
            "deviceBrand": "Apple",
            "email": "test2@gmail.com",
            "captcha": "000000",
            "password": "123456789"
            }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/registerByEmail",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);
      
    }
}
