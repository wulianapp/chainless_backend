use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, Display, PartialEq)]
pub enum BtcGradeStatus {
    /// 未绑定
    NotBind,
    /// 待评级
    PendingCalculate,
    /// 已评级
    Calculated,
    /// 已确认绑定
    Reconfirmed,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Airdrop {
    pub user_id: u32,
    pub account_id: Option<String>,
    pub invite_code: String,
    pub predecessor_user_id: u32,
    pub predecessor_account_id: String,
    pub btc_address: Option<String>,
    pub btc_level: Option<u8>,
    pub btc_grade_status: BtcGradeStatus,
    pub ref_btc_address: Option<String>,
}
