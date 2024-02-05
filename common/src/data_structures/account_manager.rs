use serde_derive::Serialize;

/***
 id serial  primary key,
    phone_number text collate pg_catalog."default" not null,
    email text collate pg_catalog."default" not null,
    pwd_hash text collate pg_catalog."default" not null,
    status smallint not null,
    predecessor text,
    verified boolean not null,
    invite_code text collate pg_catalog."default" not null,
    account_ids text[] not null,
*/
#[derive(Serialize, Debug, Clone)]
pub struct UserInfo {
    pub phone_number: String,
    pub email: String,
    pub pwd_hash: String,
    pub predecessor: Option<u32>,
    pub status: u8,
    pub verified: bool,
    pub invite_code: String,
    pub account_ids: Vec<String>,
}
impl Default for UserInfo {
    fn default() -> Self {
        UserInfo {
            phone_number: "".to_string(),
            email: "".to_string(),
            pwd_hash: "".to_string(),
            predecessor: None,
            status: 0,
            verified: false,
            invite_code: "".to_string(),
            account_ids: vec![],
        }
    }
}
