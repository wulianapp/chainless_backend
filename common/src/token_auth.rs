use std::fmt::format;
use std::ops::Div;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation, Algorithm};
use actix_web::{web, App, HttpServer, HttpResponse, Responder, HttpRequest, Error};
use actix_web::cookie::time::OffsetDateTime;
use actix_web::dev::ServiceRequest;
use actix_web::http::header;
use super::utils::time::get_unix_time;
use crate::token_auth;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iat: u64,
    exp: u64,
}

impl Claims {

    pub fn new(sub: String, iat: u64, exp: u64) -> Self {
        Self { sub, iat, exp }
    }
}


// todo: Secret key for JWT,setup by env or config
const SECRET_KEY: &[u8] = b"your_secret_key";
const DAY15: u64 =  15 * 24 * 60 * 60 * 1000;
//convenient for test
const YEAR100: u64 =  100 * 365 * 24 * 60 * 60 * 1000;
pub fn create_jwt(user_id: u32) -> String {
    let sub = format!("{}",user_id);
    let iat = get_unix_time();
    let exp = iat + DAY15;

    let claims = Claims::new(sub, iat, exp);

     jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    ).unwrap()
}

fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    ).map(|data| data.claims)
}


pub fn validate_credentials(req: &HttpRequest) -> Result<String, String> {
    let auth_header = req.headers().get(header::AUTHORIZATION)
        .ok_or("No Authorization header".to_string())?;

    let auth_str = auth_header.to_str().map_err(|_err| "Token is invalid".to_string())?;
    if auth_str.starts_with("Bearer ") {
        let token = &auth_str["Bearer ".len()..];
        let claim_dat = validate_jwt(token).map_err(|_err| "Invalid token signature".to_string())?;
        if get_unix_time() > claim_dat.exp {
            Err("Token has expired.".to_string())
        }else {
            Ok(claim_dat.sub)
        }
    } else {
        Err("Token is invalid or malformed".to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_login_auth() {
        let token = create_jwt(1);
        println!("res {}",token);
    }

}
