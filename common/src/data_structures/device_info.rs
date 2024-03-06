use serde_derive::{Deserialize, Serialize};
use super::*;

#[derive(Deserialize, Serialize, Debug,PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub user_id: u32,
    pub state: DeviceState,
    pub hold_pubkey: String,
    pub brand: String,
    pub holder_confirm_saved: bool,
}
