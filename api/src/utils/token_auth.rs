use actix_web::HttpRequest;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use models::account_manager::{UserFilter, UserInfoEntity};
use models::{PsqlOp};
use serde::{Deserialize, Serialize};

use actix_web::http::header;

use common::error_code::BackendError::Authorization;
use common::error_code::{BackendError};
use common::prelude::*;
use common::utils::time::{now_millis};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Claims {
    user_id: u32,
    version: u32,
    device_id: String,
    device_brand: String,
    iat: u64,
    exp: u64,
}

impl Claims {
    pub fn new(
        user_id: u32,
        version: u32,
        device_id: &str,
        device_brand: &str,
        iat: u64,
        exp: u64,
    ) -> Self {
        Self {
            user_id,
            version,
            device_id: device_id.to_owned(),
            device_brand: device_brand.to_owned(),
            iat,
            exp,
        }
    }
}

pub fn create_jwt(
    user_id: u32,
    version: u32,
    device_id: &str,
    device_brand: &str,
) -> Result<String, BackendError> {
    let iat = now_millis();

    let exp = iat + TOKEN_EXPAIRE_TIME;

    let claims = Claims::new(user_id, version, device_id, device_brand, iat, exp);

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TOKEN_SECRET_KEY.as_bytes()),
    )
    .map_err(|e| e.to_string().into())
}

fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(TOKEN_SECRET_KEY.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

pub async fn validate_credentials(
    req: &HttpRequest,
) -> Result<(u32, u32, String, String), BackendError> {
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
            let user_info = UserInfoEntity::find_single(UserFilter::ById(&claim_dat.user_id))
                .await
                .map_err(|err| {
                    if err.to_string().contains("DBError::DataNotFound") {
                        WalletError::MainAccountNotExist(err.to_string()).into()
                    } else {
                        BackendError::InternalError(err.to_string())
                    }
                })?
                .into_inner();

            if claim_dat.version != user_info.token_version {
                Err(Authorization("TokenVersionInvalid".to_string()))?
            }

            Ok((
                claim_dat.user_id,
                claim_dat.version,
                claim_dat.device_id.clone(),
                claim_dat.device_brand,
            ))
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
        let token = create_jwt(1, 1, "", "huawei").unwrap();
        println!("res {}", token);
    }
}
