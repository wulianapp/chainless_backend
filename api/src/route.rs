//! wulian app backend service

#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;

pub mod account_manager;
pub mod airdrop;
pub mod bridge;
pub mod general;
pub mod newbie_reward;
pub mod utils;
pub mod wallet;

use actix_cors::Cors;
use actix_web::{http, middleware, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    common::log::init_logger();
    let service: String = format!("0.0.0.0:{}", common::env::CONF.api_port);

    HttpServer::new(move || {
        //let auth = HttpAuthentication::bearer(token_auth::validate_credentials);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    //.supports_credentials()
                    .allow_any_header()
                    //.allowed_origin("127.0.0.1")
                    //.send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .configure(account_manager::configure_routes)
            .configure(wallet::configure_routes)
            .configure(bridge::configure_routes)
    })
    .bind(service)?
    .run()
    .await
}
