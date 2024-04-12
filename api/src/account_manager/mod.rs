#![feature(async_closure)]

//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};

use serde::{Deserialize, Serialize};
use tracing::{debug, Level};

//use captcha::{ContactType, VerificationCode};

use crate::utils::respond::gen_extra_respond;



/**
 * @api {post} /accountManager/getCaptchaWithoutToken 未登陆时申请验证码,
 * @apiVersion 0.0.1
 * @apiName GetCaptchaWithoutToken
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   用户设备ID,也是测试服务的验证码返回值
 * @apiBody {String} contact 用户联系方式 手机 +86 18888888888 or 邮箱 test000001@gmail.com
 * @apiBody {String="Register","Login","ResetLoginPassword"} kind 验证码类型，测试网生成的验证码为000000
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {string=0,1,2,2002,2003,2004,2005} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError,RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptchaWithoutTokenRequest {
    device_id: String,
    contact: String,
    kind: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/getCaptchaWithoutToken")]
async fn get_captcha_without_token(request_data: web::Json<GetCaptchaWithoutTokenRequest>) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_captcha::without_token_req(request_data.into_inner()))
}


/**
 * @api {post} /accountManager/getCaptchaWithToken 登陆后申请验证码
 * @apiVersion 0.0.1
 * @apiName GetCaptchaWithToken
 * @apiGroup AccountManager
 * @apiBody {String="PreSendMoneyToBridge","SetSecurity","UpdateSecurity","ResetLoginPassword","PreSendMoney","PreSendMoneyToSub","ServantSwitchMaster","NewcomerSwitchMaster"} kind 验证码类型，测试网生成的验证码为000000
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {string=0,1,2,2002,2003,2004,2005} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError,RequestParamInvalid,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect,PhoneOrEmailIncorrect} msg
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetCaptchaWithTokenRequest {
    contact:String,
    kind: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/getCaptchaWithToken")]
async fn get_captcha_with_token(request: HttpRequest,
    request_data: web::Json<GetCaptchaWithTokenRequest>) 
-> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::get_captcha::with_token_req(request,request_data.into_inner()))
}

/**
 * @api {get} /accountManager/contactIsUsed 检查账户是否已被使用
 * @apiVersion 0.0.1
 * @apiName contactIsUsed
 * @apiGroup AccountManager
 * @apiBody {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/contactIsUsed?contact=test000001@gmail.com"
 * @apiSuccess {string=0,1,} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {bool} data                result.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/contactIsUsed
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactIsUsedRequest {
    contact: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/accountManager/contactIsUsed")]
async fn contact_is_used(
    //request_data: web::Json<ContactIsUsedRequest>,
    request_data: web::Query<ContactIsUsedRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::contact_is_used::req(request_data.into_inner()))
}





/**
 * @api {get} /accountManager/checkCaptcha 验证验证码
 * @apiVersion 0.0.1
 * @apiName CheckCaptcha
 * @apiGroup AccountManager
 * @apiBody {String} contact   邮箱或者手机号
 * @apiBody {String} captcha   验证码值
 * @apiBody {String="Register","Login","ResetLoginPassword","SetSecurity","ResetLoginPassword","PreSendMoney","PreSendMoneyToSub","ServantSwitchMaster","NewcomerSwitchMaster"} usage    验证码用途

 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/contactIsUsed?contact=test000001@gmail.com"
 * @apiSuccess {string=0,1,} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {bool} data                result.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/contactIsUsed
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CheckCaptchaRequest {
    contact: String,
    captcha: String,
    usage: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/accountManager/checkCaptcha")]
async fn check_captcha(
    request_data: web::Query<CheckCaptchaRequest>,
) -> impl Responder {
    debug!("request_data {}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::check_captcha::req(request_data.into_inner()))
}

/**
 * @api {get} /accountManager/userInfo 用户账号详情
 * @apiVersion 0.0.1
 * @apiName userInfo
 * @apiGroup AccountManager
 * @apiBody {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/userInfo"
 * @apiSuccess {string=0,1,} status_code         status code.
 * @apiSuccess {string=Successfully,InternalError} msg
 * @apiSuccess {object} data                user_info
 * @apiSuccess {number} data.id                    用户id
 * @apiSuccess {string} data.phone_number         用户手机号
 * @apiSuccess {string} data.email                用户邮箱
 * @apiSuccess {string} data.anwser_indexes            安全问题的序列信息
 * @apiSuccess {bool} data.is_frozen                是否冻结
 * @apiSuccess {number} data.predecessor              邀请者ID
 * @apiSuccess {number} data.laste_predecessor_replace_time     上次更换邀请者的时间
 * @apiSuccess {string} data.invite_code              用户自己的邀请码
 * @apiSuccess {bool} data.kyc_is_verified          是否kyc
 * @apiSuccess {bool} data.secruity_is_seted        是否进行安全设置
 * @apiSuccess {string} data.main_account             主钱包id
 * @apiSuccess {string} data.role                   当前的角色
 * @apiSuccess {string} [data.name]                       kyc实名名字
 * @apiSuccess {string} [data.birthday]                   出生日期
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/userInfo
 */

type UserInfoRequest = ContactIsUsedRequest;
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[get("/accountManager/userInfo")]
async fn user_info(request: HttpRequest) -> impl Responder {
    //debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::user_info::req(request).await)
}

/**
  __apiBody {String} encryptedPrikey    私钥两次私钥加密后密文的拼接
  ———apiBody {String} pubkey     公钥的hex表达
* @api {post} /accountManager/registerByEmail 通过邮箱注册账户
* @apiVersion 0.0.1
* @apiName registerByEmail
* @apiGroup AccountManager
* @apiBody {String} deviceId     设备ID
* @apiBody {String} deviceBrand  手机型号 Huawei-P20
* @apiBody {String} email        邮箱 test000001@gmail.com
* @apiBody {String} captcha      验证码
* @apiBody {String} password     登录密码
* @apiBody {String} [predecessorInviteCode]   推荐人的邀请码
* @apiExample {curl} Example usage:
    curl -X POST http://120.232.251.101:8066/accountManager/registerByEmail -H "Content-Type: application/json" -d
  '{"deviceId": "123","email": "test000001@gmail.com","captcha":"000000","password":"123456789","encryptedPrikey": "123",
   "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee3e"}'
* @apiSuccess {string=0,1,2002,2003,2004} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/accountManager/registerByEmail
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterByEmailRequest {
    device_id: String,
    device_brand: String,
    email: String,
    captcha: String,
    password: String,
    //第一个账户肯定没有predecessor
    predecessor_invite_code: Option<String>,
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/registerByEmail")]
async fn register_by_email(request_data: web::Json<RegisterByEmailRequest>) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
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
*    curl -X POST http://120.232.251.101:8066/accountManager/registerByPhone -H "Content-Type: application/json" -d
  '{"deviceId": "123","phoneNumber": "+86 13682000011","captcha":"000000","password":"123456789","encryptedPrikey": "123",
   "pubkey": "7d2e7d073257358277821954b0b0d173077f6504e50a8fefe3ac02e2bff9ee33","predecessorInviteCode":"1"}'
* @apiSuccess {string=0,1,2002,2003,2004} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect} msg
* @apiSuccess {string} data                jwt token.
* @apiSampleRequest http://120.232.251.101:8066/accountManager/registerByEmail
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
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::register::by_phone::req(request_data.into_inner()).await)
}

/**
 * @api {post} /accountManager/login 登录
 * @apiVersion 0.0.1
 * @apiName login
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   设备ID
 * @apiBody {String} deviceBrand  手机型号 Huawei-P20
 * @apiBody {String} contact    例如 phone +86 18888888888 or email test000001@gmail.com
 * @apiBody {String} password   密码 1234
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8066/accountManager/login -H "Content-Type: application/json" -d
 *  '{"deviceId": "1234","contact": "test000001@gmail.com","password":"123456789"}'
* @apiSuccess {string=0,1,2012,2009} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,AccountLocked,PasswordIncorrect} msg
 * @apiSuccess {string} data                jwt token.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/login
 */
#[derive(Deserialize, Serialize, Clone, Debug)]
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
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::login::req(request_data.into_inner()).await)
}



/**
 * @api {post} /accountManager/loginByCaptcha   通过验证码登录
 * @apiVersion 0.0.1
 * @apiName LoginByCaptcha
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   设备ID
 * @apiBody {String} deviceBrand  手机型号 Huawei-P20
 * @apiBody {String} contact    例如 phone +86 18888888888 or email test000001@gmail.com
 * @apiBody {String} captcha   验证码 000000
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8066/accountManager/login -H "Content-Type: application/json" -d
 *  '{"deviceId": "1234","contact": "test000001@gmail.com","password":"123456789"}'
* @apiSuccess {string=0,1,2012,2009} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,AccountLocked,PasswordIncorrect} msg
 * @apiSuccess {string} data                jwt token.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/login
 */
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginByCaptchaRequest {
    device_id: String,
    device_brand: String,
    contact: String,
    captcha: String,
}
#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/loginByCaptcha")]
async fn login_by_captcha(request_data: web::Json<LoginByCaptchaRequest>) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::login::req_by_captcha(request_data.into_inner()).await)
}

/**
* @api {post} /accountManager/resetPassword  重置登录密码
* @apiVersion 0.0.1
* @apiName ResetPassword
* @apiGroup AccountManager
* @apiBody {String} deviceId     设备ID
* @apiBody {String} contact      手机或邮箱 +86 18888888888 or email test000001@gmail.com
* @apiBody {String} newPassword  新密码  "abcd"
* @apiBody {String} captcha      验证码  "123456"
* @apiExample {curl} Example usage:
  curl -X POST http://120.232.251.101:8066/accountManager/resetPassword -H "Content-Type: application/json"
 -d '{"deviceId": "123","contact": "test000001@gmail.com","captcha":"287695","newPassword":"123456788"}'
* @apiSuccess {string=0,1,2002,2003,2004} status_code         status code.
* @apiSuccess {string=Successfully,InternalError,CaptchaNotFound,CaptchaExpired,CaptchaIncorrect} msg
* @apiSuccess {string} data                nothing.
* @apiSampleRequest http://120.232.251.101:8066/accountManager/resetPassword
*/
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    contact: String,
    captcha: String,
    new_password: String,
    device_id: String
}

#[tracing::instrument(skip_all,fields(trace_id = common::log::generate_trace_id()))]
#[post("/accountManager/resetPassword")]
async fn reset_password(
    req: HttpRequest,
    request_data: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(handlers::reset_password::req(req, request_data).await)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg
        //.service(get_captcha)
        .service(contact_is_used)
        .service(register_by_email)
        .service(register_by_phone)
        .service(login)
        .service(login_by_captcha)
        .service(user_info)
        .service(get_captcha_with_token)
        .service(get_captcha_without_token)
        .service(check_captcha)
        .service(reset_password);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_service_call;
    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::{test, App, Error};
    use std::env;
    use tests::handlers::user_info::UserInfoTmp;

    use crate::utils::respond::BackendRespond;
    async fn clear_contract(_account_id: &str) {
        let cli = blockchain::ContractClient::<blockchain::multi_sig::MultiSig>::new().unwrap();
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
        let payload =
            r#"{ "deviceId": "000000", "contact": "test000001@gmail.com","kind": "Register" }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/getCaptcha",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);
        assert_eq!(res.status_code, 0);

        //register
        let payload = r#"
            { 
            "deviceId": "000000",
            "deviceBrand": "Apple",
            "email": "test000001@gmail.com",
            "captcha": "000001",
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
        assert_eq!(res.status_code, 0);

        //login
        let payload = r#"
            { "deviceId": "000000",
             "deviceBrand": "Apple",
            "contact": "test000001@gmail.com",
             "password": "123456789"
            }"#;
        let login_res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/login",
            Some(payload),
            None::<String>
        );
        println!("{:?}", login_res.data);
        assert_eq!(login_res.status_code, 0);

        //check contact if is used
        let res: BackendRespond<bool> = test_service_call!(
            service,
            "get",
            "/accountManager/contactIsUsed?contact=test000001@gmail.com",
            None::<String>,
            None::<String>
        );
        println!("{:?}", res.data);
        assert_eq!(res.status_code, 0);

        let payload = r#"{ "deviceId": "000000", "contact": "test000001@gmail.com","kind": "resetPassword" }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/getCaptcha",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);
        assert_eq!(res.status_code, 0);

        let payload = r#"
        { "deviceId": "000000",
         "captcha": "000001",
         "contact": "test000001@gmail.com",
         "newPassword": "new123456789"
        }
        "#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/resetPassword",
            Some(payload),
            Some(login_res.data.unwrap())
        );
        println!("{:?}", res.msg);
        assert_eq!(res.status_code, 0);

        let payload = r#"
        { "deviceId": "000000",
         "deviceBrand": "Apple",
        "contact": "test000001@gmail.com",
         "password": "new123456789"
        }"#;
        let res: BackendRespond<String> = test_service_call!(
            service,
            "post",
            "/accountManager/login",
            Some(payload),
            None::<String>
        );
        println!("{:?}", res.data);
        assert_eq!(res.status_code, 0);

        let res: BackendRespond<UserInfoTmp> = test_service_call!(
            service,
            "get",
            "/accountManager/userInfo",
            None::<String>,
            Some(res.data.unwrap())
        );
        println!("{:?}", res.data);
    }
}
