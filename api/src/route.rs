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
use actix_web::{
    error, get, http, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use std::env;

use actix_web_httpauth::middleware::HttpAuthentication;
use blockchain::coin::decode_coin_transfer;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::wallet::{CoinTransaction, CoinTxStatus, SecretKeyType};
use common::error_code::{AccountManagerError, WalletError};
use log::info;
use models::account_manager::UserFilter;
use models::coin_transfer::CoinTxFilter;
use models::wallet::{get_wallet, WalletFilter};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use lettre::transport::smtp::client::CertificateStore::Default;
use common::http::gen_extra_respond;

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
            .service(wallet::upload_tx_signed_data)
            //.service(wallet::backup_secret_keys)


    })
    .bind(service)?
    .run()
    .await
}
