use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use actix_web::HttpRequest;
use common::error_code::BackendError;
use common::error_code::BackendError::InternalError;
use common::http::{BackendRes, token_auth};
use crate::wallet::{NewMasterRequest, PutPendingPubkeyRequest};
lazy_static! {
    static ref PENDING_KEYS: Mutex<HashMap<u32, Vec<(String,String)>>> = Mutex::new(HashMap::new());
}

pub fn get_user_pending_keys(user_id:u32) -> Result<Vec<(String,String)>,BackendError> {
    let pending_keys_storage = PENDING_KEYS
        .lock()
        .map_err(|e| InternalError(e.to_string()))?
        .get(&user_id)
        .unwrap_or(&Vec::new())
        .to_vec();
    Ok(pending_keys_storage)
}

pub async fn req_get(req: HttpRequest) -> BackendRes<Vec<String>> {
    let user_id = token_auth::validate_credentials(&req)?;
    let keys = get_user_pending_keys(user_id)?;
    let pending_pubkey: Vec<String> = keys.iter().map(|x| x.1.to_owned()).collect();
    Ok(Some(pending_pubkey))
}

pub async fn req_put(req: HttpRequest, request_data: PutPendingPubkeyRequest) -> BackendRes<String> {
    let user_id = token_auth::validate_credentials(&req)?;
    let PutPendingPubkeyRequest{encrypted_prikey,pubkey} = request_data;
    //if key is already used，should throw error
    let pending_keys_storage = &mut PENDING_KEYS
        .lock()
        .map_err(|e| InternalError(e.to_string()))?;
    pending_keys_storage.entry(user_id).or_insert(vec![]).push((encrypted_prikey,pubkey));
    Ok(None::<String>)
}
//remove的逻辑，在其他接口，添加从设备、更换主设备的时候的进行处理
