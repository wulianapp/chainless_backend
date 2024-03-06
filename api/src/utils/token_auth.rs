use actix_web::HttpRequest;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use common::error_code::BackendError;
use common::error_code::BackendError::Authorization;
use common::utils::time::{now_millis, DAY15, YEAR100};
use actix_web::http::header;
use common::env::ServiceMode;
use common::utils::math::gen_random_verify_code;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Claims {
    user_id: u32,
    device_id: String,
    device_brand: String,
    iat: u64,
    exp: u64,
}

impl Claims {
    pub fn new(user_id: u32, device_id: &str,device_brand: &str, iat: u64, exp: u64) -> Self {
        Self {
            user_id,
            device_id: device_id.to_owned(),
            device_brand: device_brand.to_owned(),
            iat,
            exp,
        }
    }
}

// todo: Secret key for JWT,setup by env or config
const SECRET_KEY: &[u8] = b"your_secret_key";

pub fn create_jwt(user_id: u32, device_id: &str,device_brand:&str) -> String {
    let iat = now_millis();

    let exp = if common::env::CONF.service_mode != ServiceMode::Product
        && common::env::CONF.service_mode != ServiceMode::Dev
    {
        iat + YEAR100
    } else {
        iat + DAY15
    };

    let claims = Claims::new(user_id, device_id, device_brand,iat, exp);

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .unwrap()
}

fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

pub fn validate_credentials(req: &HttpRequest) -> Result<u32, BackendError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(Authorization("No Authorization header".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_err| Authorization("Token is invalid".to_string()))?;
    if auth_str.starts_with("bearer ") || auth_str.starts_with("Bearer ") {
        let token = &auth_str["bearer ".len()..];
        let claim_dat = validate_jwt(token)
            .map_err(|_err| Authorization("Invalid token signature".to_string()))?;
        if now_millis() > claim_dat.exp {
            Err(Authorization("Token has expired.".to_string()))?
        } else {
            Ok(claim_dat.user_id)
        }
    } else {
        Err(Authorization("Token is invalid or malformed".to_string()))?
    }
}

pub fn validate_credentials2(req: &HttpRequest) -> Result<(u32,String,String), BackendError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(Authorization("No Authorization header".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_err| Authorization("Token is invalid".to_string()))?;
    if auth_str.starts_with("bearer ") || auth_str.starts_with("Bearer ") {
        let token = &auth_str["bearer ".len()..];
        let claim_dat = validate_jwt(token)
            .map_err(|_err| Authorization("Invalid token signature".to_string()))?;
        if now_millis() > claim_dat.exp {
            Err(Authorization("Token has expired.".to_string()))?
        } else {
            Ok((claim_dat.user_id,claim_dat.device_id.clone(),claim_dat.device_brand))
        }
    } else {
        Err(Authorization("Token is invalid or malformed".to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_login_auth() {
        let token = create_jwt(1, "","huawei");
        println!("res {}", token);
    }
}
