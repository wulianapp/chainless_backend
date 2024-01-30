use std::fmt::Display;
use actix_web::{HttpResponse, Responder};
use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
//use anyhow::Result;

use crate::error_code::{ApiError, ChainLessError, WalletError};

pub mod token_auth;

pub type ApiRes<D,E = ApiError>  = Result<Option<D>, E>;

#[derive(Deserialize, Serialize)]
pub struct BackendRespond<T: Serialize> {
    status_code: u16,
    msg: String,
    //200 default success
    data: T,
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

pub fn generate_error_respond<E: ChainLessError + Display>(error: E) -> HttpResponse {
    return HttpResponse::Ok().json(BackendRespond {
        msg: error.to_string(),
        status_code: error.code(),
        data: "".to_string(),
    });
}

pub fn gen_extra_respond<D: Serialize,E: ChainLessError + Display>(inner_res: ApiRes<D,E>) -> impl Responder {
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