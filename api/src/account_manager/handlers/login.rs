use std::collections::HashMap;
use std::sync::Mutex;

use common::error_code::AccountManagerError::{
    AccountLocked, PasswordIncorrect, PhoneOrEmailNotRegister,
};

use crate::account_manager::LoginRequest;
use common::http::{token_auth, BackendRes};
use common::utils::time::{now_millis, MINUTE30};
use models::{account_manager, PsqlOp};
use models::account_manager::UserFilter;

lazy_static! {
    static ref LOGIN_RETRY: Mutex<HashMap<u32, Vec<u64>>> = Mutex::new(HashMap::new());
}

/***
fn get_retry_records(user_id:u32) -> Vec<u64>{
    let mut retry_storage = &mut LOGIN_RETRY.lock().unwrap();
    retry_storage.get(&user_id).as_ref().map(|&x| x.to_owned()).unwrap_or(vec![])
}

fn clear_retry_times(user_id:u32) {
    let mut retry_storage = &mut LOGIN_RETRY.lock().unwrap();
    retry_storage.entry(user_id).or_insert(vec![]);
}
*/
fn record_once_retry(user_id: u32) {
    let retry_storage = &mut LOGIN_RETRY.lock().unwrap();
    let now = now_millis();
    retry_storage.entry(user_id).or_insert(vec![]).push(now);
}

fn is_locked(user_id: u32) -> bool {
    let retry_storage = &mut LOGIN_RETRY.lock().unwrap();
    if let Some(records) = retry_storage.get(&user_id) {
        if records.len() >= 5 {
            if now_millis() <= *records.last().unwrap() + MINUTE30 {
                true
            } else {
                //clear retry records
                retry_storage.entry(user_id).or_insert(vec![]);
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn req(request_data: LoginRequest) -> BackendRes<String> {
    let LoginRequest {
        device_id,
        contact,
        password,
    } = request_data;
    //let user_at_stored = account_manager::get_user(UserFilter::ByPhoneOrEmail(contact))?.ok_or(PhoneOrEmailNotRegister)?;
    let user_at_stored = account_manager::UserInfoView::find_single(UserFilter::ByPhoneOrEmail(contact))?;

    if is_locked(user_at_stored.id) {
        Err(AccountLocked)?;
    }

    if password != user_at_stored.user_info.login_pwd_hash {
        record_once_retry(user_at_stored.id);
        Err(PasswordIncorrect)?;
    }
    //generate auth token
    let token = token_auth::create_jwt(user_at_stored.id, device_id);
    Ok(Some(token))
}
