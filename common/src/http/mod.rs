use actix_web::{HttpResponse, Responder};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
//use anyhow::Result;

use crate::error_code::{BackendError, ErrorCode};

pub mod token_auth;

pub type BackendRes<D, E = BackendError> = Result<Option<D>, E>;

#[derive(Deserialize, Serialize)]
pub struct BackendRespond<T: Serialize> {
    pub status_code: u16,
    pub msg: String,
    //200 default success
    pub data: T,
}

pub fn generate_ok_respond(info: Option<impl Serialize>) -> HttpResponse {
    if let Some(data) = info {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully".to_string(),
            status_code: 0u16,
            data,
        })
    } else {
        HttpResponse::Ok().json(BackendRespond {
            msg: "successfully".to_string(),
            status_code: 0u16,
            data: "".to_string(),
        })
    }
}

pub fn generate_error_respond<E: ErrorCode + Display>(error: E) -> HttpResponse {
    return HttpResponse::Ok().json(BackendRespond {
        msg: error.to_string(),
        status_code: error.code(),
        data: "".to_string(),
    });
}

pub fn gen_extra_respond<D: Serialize, E: ErrorCode + Display>(
    inner_res: BackendRes<D, E>,
) -> impl Responder {
    match inner_res {
        Ok(data) => generate_ok_respond(data),
        Err(error) => {
            if error.to_string().contains("Authorization") {
                HttpResponse::Unauthorized().json(error.to_string())
            } else {
                generate_error_respond(error)
            }
        }
    }
}

#[macro_export]
macro_rules! test_service_call {
    ( $service:expr,$method:expr,$api:expr,$payload:expr,$token:expr) => {{
        let mut parameters = if $method == "post" {
            test::TestRequest::post()
                .uri($api)
                .insert_header(header::ContentType::json())
        } else {
            test::TestRequest::get().uri($api)
        };

        if let Some(data) = $payload {
            parameters = parameters.set_payload(data);
        };

        if let Some(data) = $token {
            parameters =
                parameters.insert_header((header::AUTHORIZATION, format!("bearer {}", data)));
        };

        let req = parameters.to_request();
        let body = test::call_and_read_body(&$service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("body_str {}", body_str);
        serde_json::from_str::<_>(&body_str).unwrap()
    }};
}
