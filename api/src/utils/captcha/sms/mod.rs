//! 短信发送统一入口

mod cmtelecom;
mod smsbao;

use anyhow::{anyhow, Result};
use common::env::CONF;

/// 根据 `phone` 格式自动选择相应服务商发送短信。
///
/// `phone` 示例: "+86 13200001111"
///
/// 为简单避免短信自动切割成多条浪费费用，`msg` 的长度不要超过 60 字符。
///
/// `reference` 就像 JSONRPC 里的 ID，建议传个唯一值。
pub async fn send_sms(phone: &str, msg: &str, reference: &str) -> Result<()> {
    if phone.starts_with("+86 ") {
        smsbao::send_code(
            phone,
            msg,
            &CONF.sms.smsbao_username,
            &CONF.sms.smsbao_api_key,
        )
        .await
    } else {
        cmtelecom::send_code(phone, msg, reference, &CONF.sms.cmtelecom_api_key, 1).await
    }
}
