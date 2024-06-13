use actix_web::HttpRequest;

use common::error_code::BackendRes;

//use super::super::ContactIsUsedRequest;
use crate::utils::token_auth;

pub async fn req(req: HttpRequest) -> BackendRes<String> {
    let (user_id, token_version, device_id, device_brand) =
        token_auth::validate_credentials(&req).await?;
    let token =
        crate::utils::token_auth::create_jwt(user_id, token_version, &device_id, &device_brand)?;
    Ok(Some(token))
}
