use common::utils::math::gen_random_verify_code;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use anyhow::Result;

use crate::account_manager::captcha::Captcha;

enum EmailError {
    IllegalAccount,
}

fn is_valid() -> Result<(), EmailError> {
    unimplemented!()
}

pub fn send_email(code: &Captcha) -> Result<(), lettre::error::Error> {
    // 替换为您的 Gmail 邮箱地址和密码
    let email_address = "wulian2023@outlook.com";
    let email_password = "wl20230711";
    let to = code.owner.clone();
    let captcha = gen_random_verify_code();
    let captcha_str = captcha.to_string();

    // 创建电子邮件内容
    let email = Message::builder()
        .from(email_address.parse().unwrap())
        //.reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Verification Code")
        .header(ContentType::TEXT_PLAIN)
        .body(captcha_str)
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

    #[test]
    fn test_send_email() {
        /***
        let code = Captcha::new("eddy1guo@gmail.com".to_string());
        let res = send_email(&code);
        println!("res {:?}", res);

         */
    }
}
