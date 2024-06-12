//! wulian app backend service

#![allow(unused_imports)]
#![allow(dead_code)]

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

use actix_http::Payload;



use actix_cors::Cors;
use actix_web::{
    http, App, HttpServer,
};
use env_logger::Env;

use models::{general::run_api_call};
use tracing::debug;

use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

fn print_body(req: &ServiceRequest) {
    match req.parts().1 {
        Payload::H1 { payload } => {
            debug!("payload {:?}", payload)
        }
        _ => {
            //unimplemented!()
        }
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct MoreLog;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for MoreLog
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MoreLogMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MoreLogMiddleware { service }))
    }
}

pub struct MoreLogMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MoreLogMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        debug!(
            "new_requested: {} ,{}, {},{:?}",
            req.method(),
            req.path(),
            req.query_string(),
            req.head(),
        );
        print_body(&req);
        let method = req.method().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            //在tokio的本地任务和pg的连接的环境中执行api请求
            run_api_call(&method, fut).await.unwrap()
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|_| {
        println!("Custom panic hook");
    }));

    common::log::init_logger();
    let service: String = format!("0.0.0.0:{}", common::env::CONF.api_port);
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(move || {
        App::new()
            .wrap(MoreLog)
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    //.supports_credentials()
                    .allow_any_header()
                    //.allowed_origin("127.0.0.1")
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .configure(account_manager::configure_routes)
            .configure(wallet::configure_routes)
            .configure(bridge::configure_routes)
            .configure(airdrop::configure_routes)
    })
    .bind(service)?
    .run()
    .await
}
