//! 对接 smsbao.com 短信发送服务
//!
//! 运营商这边64个字符计一条短信费用，字符包含 汉字 数字 字母 标点 空格
//! 超出64个字符按照64个字符叠加计费

use std::collections::HashMap;

use anyhow::{anyhow, Result};

/// 国内短信入口
const LOCAL_ENTRY: &str = "https://api.smsbao.com/sms";
/// 国际短信入口
const GLOBAL_ENTRY: &str = "https://api.smsbao.com/wsms";

lazy_static! {
    /// 官方的错误码
    pub static ref ERROR_CODES: HashMap<String,String> = [
        ("0", "短信发送成功"),
        ("-1", "参数不全"),
        (
            "-2",
            "服务器空间不支持,请确认支持curl或者fsocket,联系您的空间商解决或者更换空间",
        ),
        ("30", "密码错误"),
        ("40", "账号不存在"),
        ("41", "余额不足"),
        ("42", "账户已过期"),
        ("43", "IP地址限制"),
        ("50", "内容含有敏感词"),
        ("51", "手机号码不正确"),
    ].map(|r|(r.0.to_string(),r.1.to_string())).into();
}

pub(crate) async fn send_code(phone: &str, msg: &str, username: &str, api_key: &str) -> Result<()> {
    let (entry, m) = if phone.starts_with("+86 ") {
        (LOCAL_ENTRY, phone.strip_prefix("+86 ").unwrap().to_owned())
    } else {
        (GLOBAL_ENTRY, phone.replace(' ', ""))
    };

    let client = reqwest::Client::new();
    let res = client
        .get(entry)
        .query(&[("u", username), ("p", api_key), ("m", &m), ("c", msg)])
        .send()
        .await?;
    match (res.status().as_u16(), res.text().await?) {
        (200, ref c) => {
            if c == "0" {
                Ok(())
            } else {
                Err(anyhow!("{} {}", c, ERROR_CODES.get(c).unwrap_or(c)))
            }
        }
        (_, c) => Err(anyhow!(c)),
    }
}
