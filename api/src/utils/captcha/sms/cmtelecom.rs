//! 对接 cmtelecom.com 短信发送服务
//!
//! 短信字符数
//!
//! GSM 字符（A-Z/0-9/空格/!@#$%^&*()_+-=<>,./?）单条短信 160 个字符，超过 160 个字
//! 符的拆分短信每条容纳 153 个字符，CM 最多能接收分成 8 段的长短信。
//!
//! Unicode 字符（中文/日文/泰文/等特殊字符）单条短信 70 个字符，超过 70 个字符的拆分
//! 短信每条容纳 67 个字符 ，CM 最多能接收分成 8 段的长短信。（注意： 只要短信里有一个字
//! 符是中文/日文/泰文/等特殊字符的话都会按照该规范计算.）
//!
//! JSON POST Error codes: https://developers.cm.com/messaging/docs/shared-features#json-post-error-codes

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const API: &str = "https://gw.cmtelecom.com/v1.0/message";

pub(crate) async fn send_code(
    phone: &str,
    msg: &str,
    reference: &str,
    api_key: &str,
    max_parts: u8,
) -> Result<()> {
    let payload = Payload {
        messages: Messages {
            authentication: Auth {
                token: api_key.into(),
            },
            msg: vec![Msg {
                from: "chainless".into(),
                body: Body {
                    content: msg.into(),
                    typ: "AUTO".into(),
                },
                min_parts: 1,
                max_parts,
                reference: reference.into(),
                to: vec![To {
                    number: format!("00{}", phone.strip_prefix('+').unwrap().replace(' ', "")),
                }],
            }],
        },
    };

    let client = reqwest::Client::new();
    let res = client.post(API).json(&payload).send().await?;
    match res.status().as_u16() {
        200 => Ok(()),
        400 => {
            let body = res.json::<ResponseBody>().await?;
            Err(anyhow!(body.messages[0].details.clone()))
        }
        _ => Err(anyhow!(res.text().await?)),
    }
}

#[derive(Serialize)]
struct Body {
    pub content: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Serialize)]
struct To {
    pub number: String,
}

#[derive(Serialize)]
struct Msg {
    pub from: String,
    pub body: Body,
    #[serde(rename = "minimumNumberOfMessageParts")]
    pub min_parts: u8,
    #[serde(rename = "maximumNumberOfMessageParts")]
    pub max_parts: u8,
    pub reference: String,
    pub to: Vec<To>,
}

#[derive(Serialize)]
struct Auth {
    #[serde(rename = "productToken")]
    pub token: String,
}

#[derive(Serialize)]
struct Messages {
    pub authentication: Auth,
    pub msg: Vec<Msg>,
}

#[derive(Serialize)]
struct Payload {
    pub messages: Messages,
}

#[derive(Deserialize)]
struct ResponseMessage {
    #[serde(rename = "messageDetails")]
    pub details: String,
}
#[derive(Deserialize)]
struct ResponseBody {
    pub messages: Vec<ResponseMessage>,
}
