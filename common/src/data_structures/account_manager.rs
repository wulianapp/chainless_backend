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
    pub main_account: String,
    pub token_version: u32,
}
