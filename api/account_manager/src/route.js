//! account manager http service

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

mod verification_code;
mod token_auth;


use std::env;
use actix_cors::Cors;
use actix_web::{error, http, Error, HttpRequest, middleware, get,
                post, web, App, HttpResponse, HttpServer, Responder};

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use actix_web_httpauth::middleware::HttpAuthentication;
use common::data_structures::account_manager::UserInfo;
use common::error_code::AccountManagerError;
use verification_code::{email, phone};
use crate::verification_code::{ContactType, VerificationCode};
use models::account_manager;

#[derive(Serialize)]
struct BackendRespond<T: Serialize> {
    status_code: u16,
    msg: String,
    //200 default success
    data: T,
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
    let GetCodeRequest { device_id, user_contact, kind } = request_data.clone();

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
    info!("get code {:?}",code);
    if contract_type == ContactType::PhoneNumber {
        //phone::send_sms(&code).unwrap()
    } else {
        //email::send_email(&code).unwrap()
    };
    //todo: code should contain code type,key is contact && code_type
    code.store().unwrap();
    //todo: delete expired

    let response = BackendRespond {
        status_code: 0,
        msg: "successfully send".to_string(),
        data: "",
    };
    HttpResponse::Ok().json(response)
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
    let VerifyCodeRequest { device_id, user_contact, kind, verification_code: code } = request_data.clone();

    //if user contact is invalided,it cann't store,and will return UserVerificationCodeNotFound in this func
    let check_res = VerificationCode::check_user_code(&user_contact, &code);
    info!("{} {} {:?}",line!(),file!(),check_res);
    let response = if let Err(error) = check_res {
        BackendRespond {
            msg: error.to_string(),
            status_code: error as u16,
            data: "".to_string(),
        }
    } else {
        BackendRespond {
            status_code: 0,
            msg: "ok ".to_string(),
            data: "".to_string(),
        }
    };
    HttpResponse::Ok().json(response)
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
    wallet_sign_strategy: String,//"<threshold>/<total>"
}

#[post("/accountManager/register")]
async fn register(request_data: web::Json<crate::RegisterRequest>) -> impl Responder {
    let crate::RegisterRequest {
        device_id,
        user_contact,
        verification_code: code,
        invitation_code,
        password,
        wallet_sign_strategy,
    } = request_data.clone();

    let check_res = VerificationCode::check_user_code(&user_contact, &code);
    info!("{} {} {:?}",line!(),file!(),check_res);
    if let Err(error) = check_res {
        return HttpResponse::Ok().json(BackendRespond {
            msg: error.to_string(),
            status_code: error as u16,
            data: "".to_string(),
        });
    };

    //check userinfo form db
    let user_at_stored = account_manager::get_by_user(&user_contact);
    if user_at_stored.is_some() {
        let error = AccountManagerError::PhoneOrEmailAlreadyRegister;
        return HttpResponse::Ok().json(BackendRespond {
            msg: error.to_string(),
            status_code: error as u16,
            data: "".to_string(),
        });
    }

    //store user info
    let mut user_info = UserInfo::default();
    user_info.email = user_contact;
    user_info.pwd_hash = password; //todo: hash it again before store
    user_info.multi_sign_strategy = wallet_sign_strategy;
    user_info.invite_code = invitation_code.unwrap_or_default();
    account_manager::single_insert(user_info).unwrap();

    HttpResponse::Ok().json(BackendRespond {
        status_code: 0,
        msg: "register successfully".to_string(),
        data: "".to_string(),
    })
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
    password: Option<String>,
    verification_code: Option<String>,
}

#[post("/accountManager/login")]
async fn login(request_data: web::Json<crate::LoginRequest>) -> impl Responder {
    let crate::LoginRequest {
        device_id,
        user_contact,
        password,
        verification_code,
    } = request_data.clone();
    let user_at_stored = account_manager::get_by_user(&user_contact);

    //check password or  verification_code
    if let Some(code) = verification_code {
        let check_res = VerificationCode::check_user_code(&user_contact, &code);
        if let Err(error) = check_res {
            return HttpResponse::Ok().json(BackendRespond {
                msg: error.to_string(),
                status_code: error as u16,
                data: "".to_string(),
            });
        }
    } else {
        if let Some(pd) = password {
            //check userinfo form db
            if user_at_stored.is_none() {
                let error = AccountManagerError::PhoneOrEmailNotRegister;
                return HttpResponse::Ok().json(BackendRespond {
                    msg: error.to_string(),
                    status_code: error as u16,
                    data: "".to_string(),
                });
            }
            if pd != user_at_stored.as_ref().unwrap().user_info.pwd_hash {
                let error = AccountManagerError::PasswordIncorrect;
                return HttpResponse::Ok().json(BackendRespond {
                    msg: error.to_string(),
                    status_code: error as u16,
                    data: "".to_string(),
                });
            }
        } else {
            let error = AccountManagerError::PasswordIncorrect;
            return HttpResponse::Ok().json(BackendRespond {
                msg: error.to_string(),
                status_code: error as u16,
                data: "".to_string(),
            });
        }
    }

    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.unwrap().id);

    HttpResponse::Ok().json(BackendRespond {
        status_code: 0,
        msg: "register successfully".to_string(),
        data: token,
    })
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
    device_id: String,
    user_contact: String,
    password: Option<String>,
    verification_code: Option<String>,
}

//async fn reset_password(request_data: web::Json<crate::ResetPassword>) -> impl Responder {
#[post("/accountManager/resetPassword")]
async fn reset_password(req:HttpRequest,request_data: web::Json<crate::ResetPassword>) -> impl Responder {
        info!("start reset_password");
    if let Err(error) = token_auth::validate_credentials(&req){
        return HttpResponse::Unauthorized().json(error.to_string());
    }
    let crate::ResetPassword {
        device_id,
        user_contact,
        password,
        verification_code,
    } = request_data.clone();
    let user_at_stored = account_manager::get_by_user(&user_contact);

    //check password or  verification_code
    if let Some(code) = verification_code {
        let check_res = VerificationCode::check_user_code(&user_contact, &code);
        if let Err(error) = check_res {
            return HttpResponse::Ok().json(BackendRespond {
                msg: error.to_string(),
                status_code: error as u16,
                data: "".to_string(),
            });
        }
    } else {
        if let Some(pd) = password {
            //check userinfo form db
            if user_at_stored.is_none() {
                let error = AccountManagerError::PhoneOrEmailNotRegister;
                return HttpResponse::Ok().json(BackendRespond {
                    msg: error.to_string(),
                    status_code: error as u16,
                    data: "".to_string(),
                });
            }
            if pd != user_at_stored.as_ref().unwrap().user_info.pwd_hash {
                let error = AccountManagerError::PasswordIncorrect;
                return HttpResponse::Ok().json(BackendRespond {
                    msg: error.to_string(),
                    status_code: error as u16,
                    data: "".to_string(),
                });
            }
        } else {
            let error = AccountManagerError::PasswordIncorrect;
            return HttpResponse::Ok().json(BackendRespond {
                msg: error.to_string(),
                status_code: error as u16,
                data: "".to_string(),
            });
        }
    }

    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.unwrap().id);
    HttpResponse::Ok().json(BackendRespond {
        status_code: 0,
        msg: "register successfully".to_string(),
        data: token,
    })
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
    let port = match env::var_os("API_PORT") {
        None => 8065u32,
        Some(mist_mode) => mist_mode.into_string().unwrap().parse::<u32>().unwrap(),
    };
    let service = format!("0.0.0.0:{}", port);

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
                    .max_age(3600)
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