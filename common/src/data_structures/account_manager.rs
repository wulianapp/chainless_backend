use serde_derive::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub phone_number: String,
    pub email: String,
    pub login_pwd_hash: String,
    pub sign_pwd_hash: String,
    //if is frozened,cannt operation anymore
    pub is_frozen: bool,
    pub predecessor: Option<u32>,
    pub laste_predecessor_replace_time: u64,
    //default is user_id
    pub invite_code: String,
    pub kyc_is_verified: bool,
    pub secruity_is_seted: bool,
    //last three time subaccounts creation
    pub create_subacc_time: Vec<u64>,
    pub main_account: String,
}
