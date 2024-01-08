use serde::Deserialize;
use serde_json::json;
use reqwest::Client;
use reqwest;
use anyhow::{Context, Result};
use common::utils::math::gen_random_verify_code;
use crate::verification_code;
use crate::verification_code::VerificationCode;

// 结构体用于反序列化 Twilio API 响应
#[derive(Deserialize)]
struct TwilioResponse {
    sid: String,
    status: String,
    // 在这里添加其他需要解析的字段
}

enum EmailError{
    IllegalAccount
}

pub async fn send_sms(code: &VerificationCode) -> Result<()> {
    // 替换为您的 Twilio 账户 SID 和认证令牌
    let account_sid = "YOUR_ACCOUNT_SID";
    let auth_token = "YOUR_AUTH_TOKEN";

    // Twilio 的手机号码和您的手机号码
    let from_number = "+1234567890";  // Twilio 的手机号码
    let to_number = "+9876543210";    // 接收验证码的手机号码

    // 生成随机验证码，这里示范生成一个随机的 6 位数字验证码
    let verification_code = gen_random_verify_code();
    let message_body = format!("Your verification code is: {}", verification_code);

    // 构建 Twilio API 请求
    let client = reqwest::Client::new();

    let response = client
        .post(format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            account_sid
        ))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .basic_auth(account_sid, Some(auth_token))
        .form(&[
            ("From", from_number),
            ("To", to_number),
            ("Body", &message_body),
        ])
        .send().await.unwrap();

    // 检查 Twilio API 的响应
    if response.status().is_success() {
        println!("Message SID: {}", response.status().as_str());
        println!("Message Status: {}", response.status().as_str());
        // 可以在这里处理其他响应字段
    } else {
        println!("Failed to send SMS: {}", response.text().await.unwrap());
    }

    Ok(())
}