//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};

use handlers::check_captcha::CheckCaptchaRequest;
use handlers::contact_is_used::ContactIsUsedRequest;
use handlers::get_captcha::GetCaptchaWithTokenRequest;
use handlers::get_captcha::GetCaptchaWithoutTokenRequest;
use handlers::get_user_device_role::GetUserDeviceRoleRequest;
use handlers::login::LoginByCaptchaRequest;
use handlers::login::LoginRequest;
use handlers::register::RegisterByEmailRequest;
use handlers::register::RegisterByPhoneRequest;
use handlers::replenish_contact::ReplenishContactRequest;
use handlers::reset_password::ResetPasswordRequest;

use tracing::debug;

//use captcha::{ContactType, VerificationCode};

use crate::utils::respond::gen_extra_respond;
use crate::utils::respond::get_lang;
use crate::utils::respond::get_trace_id;
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
 * @apiSuccess {String=0,1,2,2006,2008,2011,3008} status_code         状态码.
 * @apiSuccess {String} msg 状态信息     错误信息
 * @apiSuccess {String} data                null
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken
 */
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/getCaptchaWithoutToken")]
async fn get_captcha_without_token(
    req: HttpRequest,
    request_data: web::Json<GetCaptchaWithoutTokenRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::get_captcha::without_token_req(request_data.into_inner()).await,
    )
}

/**
 * @api {post} /accountManager/getCaptchaWithToken 登陆后申请验证码
 * @apiVersion 0.0.1
 * @apiName GetCaptchaWithToken
 * @apiGroup AccountManager
 * @apiBody {String="SetSecurity","UpdateSecurity","ServantSwitchMaster","NewcomerSwitchMaster","ReplenishContact"} kind 验证码类型，测试网生成的验证码为000000
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken -H "Content-Type: application/json" -d
 *  '{"deviceId": "abc","contact": "test000001@gmail.com","kind":"register"}'
 * @apiSuccess {String=0,1,2,2011,3008} status_code         状态码.
 * @apiSuccess {Stringt} msg                状态详情
 * @apiSuccess {String} data                null
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/getCaptchaWithoutToken
 */

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/getCaptchaWithToken")]
async fn get_captcha_with_token(
    req: HttpRequest,
    request_data: web::Json<GetCaptchaWithTokenRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::get_captcha::with_token_req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {get} /accountManager/contactIsUsed 检查账户是否已被使用
 * @apiVersion 0.0.1
 * @apiName contactIsUsed
 * @apiGroup AccountManager
 * @apiQuery {String} contact   邮箱或者手机号
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/contactIsUsed?contact=test000001@gmail.com"
 * @apiSuccess {String=0,1} status_code         状态码.
 * @apiSuccess {String} msg 状态信息
 * @apiSuccess {Object} data                            联系方式的状态.
 * @apiSuccess {bool} data.contact_is_register            是否已经注册.
 * @apiSuccess {bool} data.secruity_is_seted              是否进行安全问答.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/contactIsUsed
 */

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/accountManager/contactIsUsed")]
async fn contact_is_used(
    req: HttpRequest,
    request_data: web::Query<ContactIsUsedRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::contact_is_used::req(request_data.into_inner()).await,
    )
}

/**
* @api {get} /accountManager/checkCaptcha 验证验证码
* @apiVersion 0.0.1
* @apiName CheckCaptcha
* @apiGroup AccountManager
* @apiBody {String} contact   邮箱或者手机号
* @apiBody {String} captcha   验证码值
* @apiBody {String="Register","Login","ResetLoginPassword","SetSecurity","ResetLoginPassword","ServantSwitchMaster","NewcomerSwitchMaster"} usage    验证码用途

* @apiExample {curl} Example usage:
* curl -X GET "http://120.232.251.101:8066/accountManager/contactIsUsed?contact=test000001@gmail.com"
* @apiSuccess {String=0,1,2,2008} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {bool} data                验证码是否存在
* @apiSampleRequest http://120.232.251.101:8066/accountManager/contactIsUsed
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/accountManager/checkCaptcha")]
async fn check_captcha(
    req: HttpRequest,
    request_data: web::Query<CheckCaptchaRequest>,
) -> impl Responder {
    debug!(
        "request_data {}",
        serde_json::to_string(&request_data.0).unwrap()
    );
    gen_extra_respond(
        get_lang(&req),
        handlers::check_captcha::req(request_data.into_inner()).await,
    )
}

/**
 * @api {get} /accountManager/userInfo 用户账号详情
 * @apiVersion 0.0.1
 * @apiName userInfo
 * @apiGroup AccountManager
 * @apiHeader {String} Authorization  user's access token
 * @apiHeader {String} Request-Iddd  request-id
 * @apiExample {curl} Example usage:
 * curl -X GET "http://120.232.251.101:8066/accountManager/userInfo"
 * @apiSuccess {String=0,1,} status_code         状态码.
 * @apiSuccess {String} msg 状态信息
 * @apiSuccess {object} data                user_info
 * @apiSuccess {Number} data.id                    用户id
 * @apiSuccess {String} data.phone_number         用户手机号
 * @apiSuccess {String} data.email                用户邮箱
 * @apiSuccess {String} data.anwser_indexes            安全问题的序列信息
 * @apiSuccess {bool} data.is_frozen                是否冻结
 * @apiSuccess {Number} data.predecessor              邀请者ID
 * @apiSuccess {Number} data.laste_predecessor_replace_time     上次更换邀请者的时间
 * @apiSuccess {String} data.invite_code              用户自己的邀请码
 * @apiSuccess {bool} data.kyc_is_verified          是否kyc
 * @apiSuccess {bool} data.secruity_is_seted        是否进行安全设置
 * @apiSuccess {String} data.main_account             主钱包id
 * @apiSuccess {String=Master,Servant,Undefined} data.role                   当前的角色
 * @apiSuccess {String} [data.name]                       kyc实名名字
 * @apiSuccess {String} [data.birthday]                   出生日期
 * @apiSuccess {String} data.invite_url                   邀请链接
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/userInfo
 */

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/accountManager/userInfo")]
async fn user_info(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::user_info::req(req).await)
}

/**
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
* @apiSuccess {String=0,1,2002,2003,2004,2006,2013,2016} status_code         状态码.
* @apiSuccess {String} msg                 状态详情
* @apiSuccess {String} data                token值.
* @apiSampleRequest http://120.232.251.101:8066/accountManager/registerByEmail
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/registerByEmail")]
async fn register_by_email(
    req: HttpRequest,
    request_data: web::Json<RegisterByEmailRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::register::by_email::req(request_data.into_inner()).await,
    )
}

/**
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
* @apiSuccess {String=0,1,2002,2003,2004,2006,2013,2016} status_code         状态码.
* @apiSuccess {String} msg          状态详情
* @apiSuccess {String} data         token值.
* @apiSampleRequest http://120.232.251.101:8066/accountManager/registerByEmail
*/
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/registerByPhone")]
async fn register_by_phone(
    req: HttpRequest,
    request_data: web::Json<RegisterByPhoneRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::register::by_phone::req(request_data.into_inner()).await,
    )
}

/**
 * @api {post} /accountManager/login 通过密码登录
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
* @apiSuccess {String=0,1,2008,2009,2012} status_code         状态码.
* @apiSuccess {String} msg                  状态详情
 * @apiSuccess {String} data                token值.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/login
 */
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/login")]
//todo: rename with loginByPassword
async fn login_by_password(
    req: HttpRequest,
    request_data: web::Json<LoginRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::login::req_by_password(request_data.into_inner()).await,
    )
}

/**
 * @api {get} /accountManager/getUserDeviceRole 获取当前用户当前设备的角色信息
 * @apiVersion 0.0.1
 * @apiName GetUserDeviceRole
 * @apiGroup AccountManager
 * @apiQuery {String}        deviceId   设备id
 * @apiQuery {String}        contact    当前用户联系方式
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8066/accountManager/getUserDeviceRole -H "Content-Type: application/json" -d
 *  '{"deviceId": "1234","contact": "test000001@gmail.com","password":"123456789"}'
* @apiSuccess {String=0,1,2008} status_code         状态码.
* @apiSuccess {String} msg  状态详情
 * @apiSuccess {String=Master,Servant,Undefined} data               角色信息.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/getUserDeviceRole
 */

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[get("/accountManager/getUserDeviceRole")]
async fn get_user_device_role(
    req: HttpRequest,
    request_data: web::Query<GetUserDeviceRoleRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::get_user_device_role::req(request_data.into_inner()).await,
    )
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
* @apiSuccess {String=0,1,2002,2003,2004,2008} status_code         状态码.
* @apiSuccess {String} msg  状态详情
 * @apiSuccess {String} data                token值.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/login
 */
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/loginByCaptcha")]
async fn login_by_captcha(
    req: HttpRequest,
    request_data: web::Json<LoginByCaptchaRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::login::req_by_captcha(request_data.into_inner()).await,
    )
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
* @apiSuccess {String=0,1,2002,2003,2004,2008,3008} status_code         状态码.
* @apiSuccess {String} msg                 状态详情
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/accountManager/resetPassword
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/resetPassword")]
async fn reset_password(
    req: HttpRequest,
    request_data: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::reset_password::req(req, request_data).await,
    )
}

/**
* @api {post} /accountManager/replenishContact  补充联系方式
* @apiVersion 0.0.1
* @apiName ReplenishContact
* @apiGroup AccountManager
* @apiBody {String} contact      手机或邮箱 +86 18888888888 or email test000001@gmail.com
* @apiBody {String} captcha      验证码
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
  curl -X POST http://120.232.251.101:8066/accountManager/ -H "Content-Type: application/json"
 -d '{"deviceId": "123","contact": "test000001@gmail.com","captcha":"287695","newPassword":"123456788"}'
* @apiSuccess {String=0,1,2002,2003,2004,2008,3008} status_code         状态码.
* @apiSuccess {String} msg                 状态详情
* @apiSuccess {String} data                null
* @apiSampleRequest http://120.232.251.101:8066/accountManager/replenishContact
*/

#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/replenishContact")]
async fn replenish_contact(
    req: HttpRequest,
    request_data: web::Json<ReplenishContactRequest>,
) -> impl Responder {
    debug!("{}", serde_json::to_string(&request_data.0).unwrap());
    gen_extra_respond(
        get_lang(&req),
        handlers::replenish_contact::req(req, request_data.into_inner()).await,
    )
}

/**
 * @api {post} /accountManager/genToken   生成新的token
 * @apiVersion 0.0.1
 * @apiName GenToken
 * @apiGroup AccountManager
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8066/accountManager/ssssss -H "Content-Type: application/json" -d
 *  '{"deviceId": "1234","contact": "test000001@gmail.com","password":"123456789"}'
* @apiSuccess {String=0,1,2002,2003,2004,2008} status_code         状态码.
* @apiSuccess {String} msg  状态详情
 * @apiSuccess {String} data                token值.
 * @apiSampleRequest http://120.232.251.101:8066/accountManager/genToken
 */
#[tracing::instrument(skip_all,fields(trace_id = get_trace_id(&req)))]
#[post("/accountManager/genToken")]
async fn gen_token(req: HttpRequest) -> impl Responder {
    gen_extra_respond(get_lang(&req), handlers::gen_token::req(req).await)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        //.service(get_captcha)
        .service(contact_is_used)
        .service(register_by_email)
        .service(register_by_phone)
        .service(login_by_password)
        .service(login_by_captcha)
        .service(user_info)
        .service(get_captcha_with_token)
        .service(get_captcha_without_token)
        .service(check_captcha)
        .service(get_user_device_role)
        .service(gen_token)
        .service(replenish_contact)
        .service(reset_password);
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{test_contact_is_used, test_login, test_register, test_reset_password, test_service_call, test_user_info};
    use crate::utils::api_test::{gen_some_accounts_with_new_key,init};
    use actix_web::body::MessageBody;
    use actix_web::http::header;
    use actix_web::{test};
    use tests::handlers::contact_is_used::ContactIsUsedResponse;
    use tests::handlers::user_info::UserInfoResponse;
    use serde_json::json;

    use crate::utils::respond::BackendRespond;


    #[actix_web::test]
    async fn test_hello() {
        let app = init().await;
        let service = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get().uri("/hello/test").to_request();
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
        let service = actix_web::test::init_service(app).await;

        let (mut sender_master, _sender_servant, _sender_newcommer, _receiver) = gen_some_accounts_with_new_key();
 
        test_register!(service,sender_master);

        test_login!(service,sender_master);

        //check contact if is used
        let res = test_contact_is_used!(service,sender_master);
        println!("used_res {:?}",res);

        sender_master.user.password = "new_pwd".to_string();
        test_reset_password!(service,sender_master);


        test_login!(service,sender_master);

        let info = test_user_info!(service,sender_master);
        println!("{:?}", info);
    }
}
