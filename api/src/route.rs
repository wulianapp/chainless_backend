//#![deny(warnings)]
//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_snake_case)]

/// wulian app backend service

extern crate common;
#[macro_use]
extern crate lazy_static;

pub mod account_manager;
pub mod account_manager_v2;
pub mod airdrop;
pub mod airdrop_v2;
pub mod bridge;
pub mod bridge_v2;
pub mod general;
pub mod utils;
pub mod wallet;
pub mod wallet_v2;

use actix_http::{header, Payload};

use actix_cors::Cors;
use actix_web::{error::ErrorInternalServerError, http, App, HttpServer};
use common::log::generate_trace_id;
use env_logger::Env;

use models::general::{clean_conn, gen_db_cli};
use tracing::{debug, info};

use std::{cell::RefCell, future::{ready, Ready}, sync::Arc};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

fn print_body(req: &ServiceRequest) {
    match req.parts().1 {
        Payload::H1 { payload } => {
            debug!("body_payload {:?}", payload)
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
            req.head()
        );
        print_body(&req);
        let method = req.method().to_string();

        let fut = self.service.call(req);
        Box::pin(async move {
            let log_id = generate_trace_id();

            let (db_cli, conn_ptr) = gen_db_cli(&method).await.map_err(ErrorInternalServerError)?;
            debug!("log_id:{} : create local cli",log_id);
            //todo: 最好LOCAL_CLI的初始化放在modles模块
            models::LOCAL_CLI.scope(RefCell::new(Some(Arc::new(db_cli))), async move {
                let res = fut.await;
                //default internal error
                let default_code = header::HeaderValue::from_str("1").unwrap();
                let err_code = res.as_ref().map(|res|{
                    let value = res.headers().get(
                        header::HeaderName::from_static("chainless_status_code")
                    ).unwrap_or(&default_code);
                    value.to_str().unwrap().parse::<u16>().unwrap()
                });

                //只有post正确完成之后才commit，否则都回滚
                match (res.as_ref(),method.as_str(),err_code) {
                    (Ok(_),"POST",Ok(0)) => {
                        models::general::commit().await.map_err(ErrorInternalServerError)?;
                    },
                    _ => {
                        models::general::rollback().await.map_err(ErrorInternalServerError)?;
                    }
                };
                clean_conn(conn_ptr);
                debug!("log_id:{} : clean local cli",log_id);
                res
            }).await
            
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    common::log::init_logger();
    info!("Service Start");
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
            .configure(account_manager_v2::configure_routes)
            .configure(wallet::configure_routes)
            .configure(wallet_v2::configure_routes)
            .configure(bridge::configure_routes)
            .configure(bridge_v2::configure_routes)
            .configure(airdrop::configure_routes)
            .configure(airdrop_v2::configure_routes)

    })
    .bind(service)?
    .run()
    .await
}
