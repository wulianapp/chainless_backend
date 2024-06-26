use super::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub user_id: u32,
    pub state: DeviceState,
    pub hold_pubkey: Option<String>,
    pub brand: String,
    pub holder_confirm_saved: bool,
}
