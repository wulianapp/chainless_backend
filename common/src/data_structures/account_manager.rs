use serde_derive::Serialize;

use super::OpStatus;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub phone_number: String,
    pub email: String,
    pub login_pwd_hash: String,
    pub anwser_indexes: String,
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
    //todo: convert to Option<String>
    pub main_account: String,
    pub op_status: OpStatus,
    pub reserved_field1: String,
    pub reserved_field2: String,
    pub reserved_field3: String,
}
