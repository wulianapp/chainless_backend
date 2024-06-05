use super::SecretKeyState;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SecretStore {
    pub pubkey: String,
    pub state: SecretKeyState,
    pub user_id: u32,
    pub encrypted_prikey_by_password: String,
    pub encrypted_prikey_by_answer: String,
}
