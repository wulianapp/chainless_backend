//! account manager http service

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

mod message;

use std::env;
use actix_cors::Cors;
use actix_web::{error, http, Error, HttpRequest, middleware, get,
                post, web, App, HttpResponse, HttpServer, Responder};

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use actix_web_httpauth::middleware::HttpAuthentication;
use common::data_structures::account_manager::UserInfo;
use common::error_code::AccountManagerError;
use common::token_auth;
use models::account_manager;
use models::account_manager::UserFilter;

#[derive(Serialize)]
struct BackendRespond<T: Serialize> {
    status_code: u16,
    msg: String,
    //200 default success
    data: T,
}


fn generate_ok_respond(info: Option<impl Serialize>) -> HttpResponse{
    if let Some(data) = info{
        HttpResponse::Ok().json( BackendRespond {
            msg: "successfully ".to_string(),
            status_code: 0u16,
            data,
        })
    }else {
        HttpResponse::Ok().json( BackendRespond {
            msg: "successfully ".to_string(),
            status_code: 0u16,
            data:"".to_string(),
        })
    }
}

fn generate_error_respond(error: AccountManagerError) -> HttpResponse{
    return HttpResponse::Ok().json(BackendRespond {
        msg: error.to_string(),
        status_code: error as u16,
        data: "".to_string(),
    });
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchMessageRequest {
    user_id: String,
}

#[post("/wallet/searchMessage")]
async fn search_message(req:HttpRequest,request_data: web::Json<SearchMessageRequest>) -> impl Responder {
    let user_id = match token_auth::validate_credentials(&req){
        Ok(date) => {
            date
        }
        Err(error) => {
            return HttpResponse::Unauthorized().json(error);
        }
    };
    //message max is 10，
    //let SearchMessageRequest {user_id} = request_data.clone();

    let store = message::MESSAGE_STORE.lock().unwrap();
    let message = store.get(&user_id).map(|x| x.clone());
    generate_ok_respond(message)

}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMoneyRequest {
    tx_raw: String,
    //platform_sign_num: u8
}

#[post("/wallet/sendMoney")]
async fn send_money(req:HttpRequest, request_data: web::Json<crate::SendMoneyRequest>) -> impl Responder {
    let user_id = match token_auth::validate_credentials(&req){
        Ok(date) => {
            date
        }
        Err(error) => {
            return HttpResponse::Unauthorized().json(error);
        }
    };
    let tx = blockchain::coin::decode_coin_transfer(&request_data.tx_raw);

    let user_at_stored = account_manager::get_by_user(
        UserFilter::ById(&user_id)
    );

    //todo: check tx.from == user_at_stored.id
    /***
    let mut store = message::MESSAGE_STORE.lock().unwrap();
    store.get_mut(&user_id).map(|map|
        {
            map.pu
        }
    );

     */
    //generate_ok_respond(message)

    generate_ok_respond(None::<String>)

}


/***
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeShardSaveRequest {
    user_id: String,
    message_id: String,
}
#[post("/wallet/finalizeShardSave")]
async fn finalize_shard_save(req:HttpRequest,request_data: web::Json<crate::SearchMessageRequest>) -> impl Responder {
    if let Err(error) = token_auth::validate_credentials(&req){
        return HttpResponse::Unauthorized().json(error);
    }
    //message max is 10，
    let FinalizeShardSaveRequest {user_id,message_id} = request_data.clone();
    generate_ok_respond(None::<String>)
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeTransferRepRequest {
    user_id: String,
    message_id: String,
}
#[post("/wallet/finalizeTransferRep")]
async fn finalize_transfer_rep(req:HttpRequest,request_data: web::Json<crate::SearchMessageRequest>) -> impl Responder {
    if let Err(error) = token_auth::validate_credentials(&req){
        return HttpResponse::Unauthorized().json(error);
    }
    //message max is 10，
    let crate::FinalizeTransferRepRequest {user_id,message_id} = request_data.clone();
    generate_ok_respond(None::<String>)
}
***/

#[get("/hello/{user}")]
async fn hello_world(user: web::Path<String>) -> impl Responder {
    format!("Hello {}! id:{}", user, 10)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let global_conf = &common::env::CONF;
    let service = format!("0.0.0.0:{}", global_conf.wallet_api_port);

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
            .service(search_message)

    })
        .bind(service.as_str())?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{body, body::MessageBody as _, rt::pin, test, web, App};
    use actix_web::body::MessageBody;
    use actix_web::dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;
    use actix_web::web::Header;

    async fn init() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config=(),
            Response=ServiceResponse,
            Error=Error,
            InitError=(),
        >,
    > {
        env::set_var("SERVICE_MODE", "test");
        //models::general::table_all_clear();
        App::new()
            .service(hello_world)
            .service(search_message)
    }

    #[actix_web::test]
    async fn test_all_basal_wallet_ok(){
        let auth_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI\
        xMiIsImlhdCI6MTcwNDQxOTgwMTQzOSwiZXhwIjo0ODU4MDE5ODAxNDM5fQ.orX01767UwwOt\
        Lrp46CE3JITQmiBQeMLeoE0Xx6LPmM";

        //reset password
        let payload = r#"
        {
         "userId": "1"
        }
        "#;

        let app = init().await;
        let service = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/wallet/searchMessage")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", auth_token)))
            .set_payload(payload.to_string())
            .to_request();
        let body = test::call_and_read_body(&service, req).await.try_into_bytes().unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        //let user: BackendRespond<String> = serde_json::from_str(&body_str).unwrap();
        println!("body_str {}",body_str);
    }
}