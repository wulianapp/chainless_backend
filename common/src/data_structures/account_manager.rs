use serde_derive::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub id: u32,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub login_pwd_hash: String,
    pub anwser_indexes: String,
    //if is frozened,cannt operation anymore
    pub is_frozen: bool,
    pub kyc_is_verified: bool,
    //last three time subaccounts creation
    pub create_subacc_time: Vec<u64>,
    pub main_account: Option<String>,
    pub token_version: u32,
}
