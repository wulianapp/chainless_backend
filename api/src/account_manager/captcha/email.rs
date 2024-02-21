use common::utils::math::gen_random_verify_code;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use anyhow::Result;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use tracing::{debug, error};
use common::error_code::{BackendError, ExternalServiceError};
use common::http::BackendRes;

use crate::account_manager::captcha::Captcha;

enum EmailError {
    IllegalAccount,
}

fn is_valid() -> Result<(), EmailError> {
    unimplemented!()
}

pub fn send_email(code: &Captcha) -> BackendRes<String> {
    // 替换为您的 Gmail 邮箱地址和密码
    let email_address = "cs2-test@chainless.top";
    let email_password = "vkHyW2dvynF8YuG1xN";
    let to = code.owner.clone();
    let captcha = gen_random_verify_code().to_string();

    // 创建电子邮件内容
    let email = Message::builder()
        .from(email_address.parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Captcha")
        .header(ContentType::TEXT_PLAIN)
        .body(captcha)
        .map_err(|e| {
            error!("Email parameters error {}",e.to_string());
            ExternalServiceError::EmailCaptcha(e.to_string())
        })?;

    let creds = Credentials::new(email_address.to_owned(), email_password.to_owned());

    let tls = TlsParameters::builder("ud.1025.hk".to_owned())
        .dangerous_accept_invalid_certs(true)
        .build().map_err(|e| {
        error!("EmailCaptcha service is crashed {}",e.to_string());
        ExternalServiceError::EmailCaptcha(e.to_string())
    })?;

    let mailer = SmtpTransport::relay("ud.1025.hk")
        .map(|c| c.port(1025)) // 指定 SMTP 服务器端口号
        .map_err(|e| {
            error!("EmailCaptcha service is crashed {}",e.to_string());
            ExternalServiceError::EmailCaptcha(e.to_string())
        })?
        .tls(Tls::Required(tls))
        .credentials(creds)
        .build();

    let send_res = mailer.send(&email).map_err(|e| {
        error!("Email send message failed {}",e.to_string());
        ExternalServiceError::EmailCaptcha(e.to_string())
    })?;
    debug!("mail send res {:?}",send_res);
    Ok(None::<String>)
}

#[cfg(test)]
mod tests {
    use crate::account_manager::captcha::Captcha;
    use crate::account_manager::captcha::email::send_email;

    #[test]
    fn test_send_email_ok() {
        let code = Captcha::new("eddy1guo@gmail.com".to_string(), "1".to_string(), crate::account_manager::captcha::Usage::Register);
        let res = send_email(&code).unwrap();
        println!("res {:?}", res);
    }
}
