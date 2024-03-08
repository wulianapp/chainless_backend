use actix_web::{HttpResponse, Responder};
use common::error_code::{BackendError, ErrorCode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::debug;

pub type BackendRes<D, E = BackendError> = Result<Option<D>, E>;

#[derive(Deserialize, Serialize)]
pub struct BackendRespond<T: Serialize> {
    pub status_code: u16,
    pub msg: String,
    //200 default success
    pub data: T,
}

pub fn generate_ok_respond(info: Option<impl Serialize>) -> HttpResponse {
    debug!(
        "generate_ok_respond: {}",
        serde_json::to_string(&info).unwrap()
    );
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
    debug!("return_error_respond: {}", error.to_string());
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
