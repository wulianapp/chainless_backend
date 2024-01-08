use std::collections::HashMap;
use std::ops::Deref;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use common::utils::math::gen_random_verify_code;
use rand::Rng;
use anyhow::Result;
use crate::verification_code;
use crate::verification_code::VerificationCode;

enum EmailError{
    IllegalAccount
}

fn is_valid() -> Result<(),EmailError>{
    unimplemented!()
}

pub fn send_email(code: &VerificationCode) -> Result<(), lettre::error::Error> {
    // 替换为您的 Gmail 邮箱地址和密码
    let email_address = "wulian2023@outlook.com";
    let email_password = "wl20230711";
    let to = code.owner.clone();
    let verification_code = gen_random_verify_code();
    let verification_code_str = verification_code.to_string();

    // 创建电子邮件内容
    let email = Message::builder()
        .from(email_address.parse().unwrap())
        //.reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Verification Code")
        .header(ContentType::TEXT_PLAIN)
        .body(verification_code_str)
        .unwrap();

    let creds = Credentials::new(email_address.to_owned(), email_password.to_owned());
    let mailer = SmtpTransport::relay("smtp-mail.outlook.com")
        .map(|c| c.port(587)) // 指定 SMTP 服务器端口号
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_email() {
        let code = VerificationCode::new("eddy1guo@gmail.com".to_string());
        let res = send_email(&code);
        println!("res {:?}",res);
    }

}