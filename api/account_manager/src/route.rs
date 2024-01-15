//! account manager http service

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

mod verification_code;

use actix_cors::Cors;
use actix_web::{
    get, http, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use log::info;
use serde::{Deserialize, Serialize};

use common::data_structures::account_manager::UserInfo;
use common::error_code::AccountManagerError;
use common::token_auth;

use crate::verification_code::{ContactType, VerificationCode};
use models::account_manager;
use models::account_manager::{get_current_user_num, UserFilter};

#[derive(Serialize, Deserialize)]
struct BackendRespond<T: Serialize> {
    status_code: u16,
    msg: String,
    //200 default success
    data: T,
}

fn generate_ok_respond(info: Option<impl Serialize>) -> HttpResponse {
    if let Some(data) = info {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully ".to_string(),
            status_code: 0u16,
            data,
        })
    } else {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully ".to_string(),
            status_code: 0u16,
            data: "".to_string(),
        })
    }
}

fn generate_error_respond(error: AccountManagerError) -> HttpResponse {
    return HttpResponse::Ok().json(BackendRespond {
        msg: error.to_string(),
        status_code: error as u16,
        data: "".to_string(),
    });
}

/**
 * @api {post} /accountManager/getCode Get verify code
 * @apiVersion 0.0.1
 * @apiName getCode
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} userContact example phone +86 18888888888 or email john@gmail.com
 * @apiBody {String=101,102,103,104} kind   101 register，102 login，103 reset password，104 shortcut login
 * @apiExample {curl} Example usage:
 *   curl -X POST http://120.232.251.101:8065/accountManager/getCode -H "Content-Type: application/json" -d
 *  '{"deviceId": "John Doe","userContact": "johnexample.com","codeType":"11","kind":"22"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/getCode
 */
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct GetCodeRequest {
    device_id: String,
    user_contact: String,
    kind: String, //
}

#[post("/accountManager/getCode")]
async fn get_code(request_data: web::Json<GetCodeRequest>) -> impl Responder {
    let GetCodeRequest {
        device_id: _,
        user_contact,
        kind: _,
    } = request_data.clone();

    let contract_type = verification_code::validate(&user_contact);
    if contract_type == ContactType::Other {
        let response = BackendRespond {
            status_code: AccountManagerError::PhoneOrEmailIncorrect as u16,
            msg: AccountManagerError::PhoneOrEmailIncorrect.to_string(),
            data: GetCodeRequest::default(),
        };
        return HttpResponse::Ok().json(response);
    }
    //todo: prohibit too frequently

    let code = VerificationCode::new(user_contact);
    info!("get code {:?}", code);
    if contract_type == ContactType::PhoneNumber {
        //phone::send_sms(&code).unwrap()
    } else {
        //email::send_email(&code).unwrap()
    };
    //todo: code should contain code type,key is contact && code_type
    code.store().unwrap();
    //todo: delete expired

    generate_ok_respond(None::<String>)
}

/**
 * @api {post} /accountManager/verifyCode check verificationCode
 * @apiVersion 0.0.1
 * @apiName verifyCode
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} userContact example phone +86 18888888888 or email john@gmail.com
 * @apiBody {String} verificationCode   user's verification code for register
 * @apiBody {String=101,102,103,104} kind   101 register，102 login，103 reset password，104 shortcut login
 * @apiExample {curl} Example usage:
 * curl -X POST http://120.232.251.101:8065/accountManager/getCode -H "Content-Type: application/json" -d
 *   '{"deviceId": "John Doe","userContact": "johne@example.com","kind":"101","verificationCode":"685886"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/verifyCode
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct VerifyCodeRequest {
    device_id: String,
    user_contact: String,
    kind: String,
    verification_code: String,
}

#[post("/accountManager/verifyCode")]
async fn verify_code(request_data: web::Json<crate::VerifyCodeRequest>) -> impl Responder {
    let VerifyCodeRequest {
        device_id: _,
        user_contact,
        kind: _,
        verification_code: code,
    } = request_data.clone();

    //if user contact is invalided,it cann't store,and will return UserVerificationCodeNotFound in this func
    let check_res = VerificationCode::check_user_code(&user_contact, &code);
    info!("{} {} {:?}", line!(), file!(), check_res);
    if let Err(error) = check_res {
        generate_error_respond(error)
    } else {
        generate_ok_respond(None::<String>)
    }
}

/**
 * @api {post} /accountManager/register register account
 * @apiVersion 0.0.1
 * @apiName register
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} userContact example phone +86 18888888888 or email john@gmail.com
 * @apiBody {String} verificationCode   user's verification code for register
 * @apiBody {String} [invitationCode]   inviter's invitation code
 * @apiBody {String} password             user's password for register
 * @apiBody {String} walletSignStrategy   example "1/12"
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8065/accountManager/register -H "Content-Type: application/json" -d
 *  '{"deviceId": "John Doe","userContact": "john@example.com","verificationCode":"287695","password":"abc","walletSignStrategy":"1/12"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/register
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    device_id: String,
    user_contact: String,
    verification_code: String,
    invitation_code: Option<String>,
    password: String,
    wallet_sign_strategy: String, //"<threshold>/<total>"
}

#[post("/accountManager/register")]
async fn register(request_data: web::Json<crate::RegisterRequest>) -> impl Responder {
    let crate::RegisterRequest {
        device_id: _,
        user_contact,
        verification_code: code,
        invitation_code,
        password,
        wallet_sign_strategy,
    } = request_data.clone();

    let check_res = VerificationCode::check_user_code(&user_contact, &code);
    println!("{} {} {:?}", line!(), file!(), check_res);
    info!("{} {} {:?}", line!(), file!(), check_res);
    if let Err(error) = check_res {
        return HttpResponse::Ok().json(BackendRespond {
            msg: error.to_string(),
            status_code: error as u16,
            data: "".to_string(),
        });
    };

    //check userinfo form db
    let user_at_stored = account_manager::get_by_user(UserFilter::ByPhoneOrEmail(&user_contact));
    println!("____{:?}", user_at_stored);
    if user_at_stored.is_some() {
        let error = AccountManagerError::PhoneOrEmailAlreadyRegister;
        return generate_error_respond(error);
    }

    //store user info
    let mut user_info = UserInfo::default();
    user_info.email = user_contact;
    user_info.pwd_hash = password; //todo: hash it again before store
    user_info.multi_sign_strategy = wallet_sign_strategy;
    //invite_code is filled with user_id by default
    let default_invite_code = (get_current_user_num() + 1).to_string();
    user_info.invite_code = invitation_code.unwrap_or(default_invite_code);
    account_manager::single_insert(user_info).unwrap();

    generate_ok_respond(None::<String>)
}

/**
 * @api {post} /accountManager/login user login
 * @apiVersion 0.0.1
 * @apiName login
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} userContact example phone +86 18888888888 or email john@gmail.com
 * @apiBody {String} [password] "abcd"
 * @apiBody {String} [verification_code] "123456"
 * @apiExample {curl} Example usage:
 *    curl -X POST http://120.232.251.101:8065/accountManager/login -H "Content-Type: application/json" -d
 *  '{"deviceId": "John Doe","userContact": "john@example.com","code":"287695"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/login
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    device_id: String,
    user_contact: String,
    password: String,
}

#[post("/accountManager/login")]
async fn login(request_data: web::Json<crate::LoginRequest>) -> impl Responder {
    let crate::LoginRequest {
        device_id: _,
        user_contact,
        password,
    } = request_data.clone();
    let user_at_stored = account_manager::get_by_user(UserFilter::ByPhoneOrEmail(&user_contact));

    //check password or  verification_code
    if user_at_stored.is_none() {
        let error = AccountManagerError::PhoneOrEmailNotRegister;
        return generate_error_respond(error);
    }
    if password != user_at_stored.as_ref().unwrap().user_info.pwd_hash {
        let error = AccountManagerError::PasswordIncorrect;
        return generate_error_respond(error);
    }

    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.unwrap().id);

    generate_ok_respond(Some(token))
}

/**
 * @api {post} /accountManager/resetPassword  reset usr password
 * @apiVersion 0.0.1
 * @apiName ResetPassword
 * @apiGroup AccountManager
 * @apiBody {String} deviceId   user's device id
 * @apiBody {String} userContact example phone +86 18888888888 or email john@gmail.com
 * @apiBody {String} [password] "abcd"
 * @apiBody {String} [verificationCode] "123456"
 * @apiHeader {String} Authorization  user's access token
 * @apiExample {curl} Example usage:
 *  curl -X POST http://120.232.251.101:8065/accountManager/resetPassword
 * -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
 *   OiJIUzI1NiJ9.eyJzdWIiOiJjaGFpbmxlc3MgdXNlcmlkOiAgNCIsImlhdCI6MTcwMzk1Njk5NywiZXhw
 * IjoxNzA1MjUyOTk3fQ.usNdrNVo2oMO0rMdW62rbbooxzOKZjoji9cNN2b1I1c'
 * -d '{"deviceId": "John Doe","userContact": "john@example.com","verificationCode":"287695"}'
 * @apiSuccess {string} status_code         status code.
 * @apiSuccess {string} msg                 description of status.
 * @apiSuccess {string} data                nothing.
 * @apiSampleRequest http://120.232.251.101:8065/accountManager/resetPassword
 */
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResetPassword {
    user_contact: String,
    verification_code: String,
    new_password: String,
}

#[post("/accountManager/resetPassword")]
async fn reset_password(
    req: HttpRequest,
    request_data: web::Json<ResetPassword>,
) -> impl Responder {
    info!("start reset_password");
    let user_id = match token_auth::validate_credentials(&req) {
        Ok(date) => date,
        Err(error) => {
            return HttpResponse::Unauthorized().json(error);
        }
    };
    let ResetPassword {
        user_contact,
        verification_code,
        new_password,
    } = request_data.clone();

    //check verification_code
    let check_res = VerificationCode::check_user_code(&user_contact, &verification_code);
    if let Err(error) = check_res {
        return generate_error_respond(error);
    }

    //modify user's password  at db
    account_manager::update_password(&new_password, UserFilter::ById(user_id));
    generate_ok_respond(None::<String>)
}

#[get("/hello/{user}")]
async fn hello_world(user: web::Path<String>) -> impl Responder {
    format!("Hello {}! id:{}", user, 10)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let global_conf = &common::env::CONF;
    let service = format!("0.0.0.0:{}", global_conf.account_manage_api_port);

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
            .service(hello_world)
            .service(echo)
            .service(get_code)
            .service(verify_code)
            .service(register)
            .service(login)
            .service(reset_password)
    })
    .bind(service.as_str())?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::{body::MessageBody as _, test, App};

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
        //models::general::table_all_clear();
        App::new()
            .service(hello_world)
            .service(echo)
            .service(get_code)
            .service(verify_code)
            .service(register)
            .service(login)
            .service(reset_password)
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

        //get code
        let payload = r#"{ "deviceId": "1", "userContact": "test2@gmail.com","kind": "1" }"#;
        let req = test::TestRequest::post()
            .uri("/accountManager/getCode")
            .insert_header(header::ContentType::json())
            //.insert_header((header::AUTHORIZATION, format!("bearer {}", token_res)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("body_str {}", body_str);

        //register
        let payload = r#"
        { "deviceId": "1",
        "userContact": "test2@gmail.com",
        "verificationCode": "000000",
         "password": "123456789",
        "walletSignStrategy": "2-3"
        }
        "#;
        let req = test::TestRequest::post()
            .uri("/accountManager/register")
            .insert_header(header::ContentType::json())
            //.insert_header((header::AUTHORIZATION, format!("bearer {}", token_res)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("body_str {}", body_str);

        //login
        let payload = r#"
        { "deviceId": "1",
        "userContact": "test2@gmail.com",
         "password": "123456789"
        }
        "#;
        let req = test::TestRequest::post()
            .uri("/accountManager/login")
            .insert_header(header::ContentType::json())
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<String> = serde_json::from_str(&body_str).unwrap();
        let auth_token = user.data;
        println!("body_str {}", auth_token);

        //reset password
        let payload = r#"
        { "verificationCode": "000000",
        "userContact": "test2@gmail.com",
         "newPassword": "123456789"
        }
        "#;
        let req = test::TestRequest::post()
            .uri("/accountManager/resetPassword")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", auth_token)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let user: BackendRespond<String> = serde_json::from_str(&body_str).unwrap();
        println!("body_str {}", user.data);
    }
}
