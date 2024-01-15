use serde_derive::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct UserInfo {
    pub phone_number: String,
    pub email: String,
    pub state: u8,
    pub multi_sign_strategy: String,
    pub verified: bool,
    pub pwd_hash: String,
    pub invite_code: String,
    pub direct_invited_number: u32,
    pub ancestors: Vec<String>,
    //pub device_ids: Vec<String>,
    pub points: u32,
    pub grade: u8,
    pub fans_num: u32,
}
impl Default for UserInfo {
    //default of string type is " ",not "";
    fn default() -> Self {
        UserInfo {
            phone_number: "".to_string(),
            email: "".to_string(),
            state: 0,
            multi_sign_strategy: "".to_string(),
            verified: false,
            pwd_hash: "".to_string(),
            invite_code: "".to_string(),
            direct_invited_number: 0,
            ancestors: vec![],
            points: 0,
            grade: 1,
            fans_num: 0,
        }
    }
}
