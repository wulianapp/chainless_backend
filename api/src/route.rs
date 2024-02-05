//! account manager http service

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

pub mod account_manager;
pub mod airdrop;
pub mod general;
pub mod newbie_reward;
pub mod wallet;

use actix_cors::Cors;
use actix_web::{http, middleware, App, HttpServer};

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
            .service(account_manager::get_captcha)
            .service(account_manager::contact_is_used)
            //.service(account_manager::verify_captcha)
            .service(account_manager::reset_password)
            .service(account_manager::register_by_email)
            .service(account_manager::register_by_phone)
            .service(account_manager::login)
            .service(wallet::search_message)
            .service(wallet::pre_send_money)
            .service(wallet::direct_send_money)
            .service(wallet::react_pre_send_money)
            .service(wallet::reconfirm_send_money)
            .service(wallet::search_message_by_account_id)
    })
    .bind(service)?
    .run()
    .await
}
