use common::utils::math::gen_random_verify_code;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use anyhow::Result;
use common::error_code::BackendRes;
use common::error_code::{BackendError, ExternalServiceError};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use tracing::{debug, error};

use crate::utils::captcha::Captcha;

enum EmailError {
    IllegalAccount,
}

fn is_valid() -> Result<(), EmailError> {
    unimplemented!()
}

pub fn send_email(code: &str, to_mail: &str) -> BackendRes<String> {
    // 替换为您的 Gmail 邮箱地址和密码
    let email_address = "cs2-test@chainless.top";
    let email_password = "vkHyW2dvynF8YuG1xN";
    let content = format!(
        "[ChainLess] Your captcha is: {}, valid for 10 minutes.",
        code
    );

    let from = email_address.parse::<Mailbox>()
    .map_err(|e| e.to_string())?;
    let to = to_mail.parse::<Mailbox>()
    .map_err(|e| e.to_string())?;

    // 创建电子邮件内容
    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("[ChainLess] Captcha")
        .header(ContentType::TEXT_PLAIN)
        .body(content)
        .map_err(|e| {
            error!("Email parameters error {}", e.to_string());
            ExternalServiceError::EmailCaptcha(e.to_string())
        })?;

    let creds = Credentials::new(email_address.to_owned(), email_password.to_owned());

    let tls = TlsParameters::builder("ud.1025.hk".to_owned())
        .dangerous_accept_invalid_certs(true)
        .build()
        .map_err(|e| {
            error!("EmailCaptcha service is crashed {}", e.to_string());
            ExternalServiceError::EmailCaptcha(e.to_string())
        })?;

    let mailer = SmtpTransport::relay("ud.1025.hk")
        .map(|c| c.port(1025)) // 指定 SMTP 服务器端口号
        .map_err(|e| {
            error!("EmailCaptcha service is crashed {}", e.to_string());
            ExternalServiceError::EmailCaptcha(e.to_string())
        })?
        .tls(Tls::Required(tls))
        .credentials(creds)
        .build();

    let send_res = mailer.send(&email).map_err(|e| {
        error!("Email send message failed {}", e.to_string());
        ExternalServiceError::EmailCaptcha(e.to_string())
    })?;
    debug!("mail send res {:?}", send_res);
    Ok(None::<String>)
}

#[cfg(test)]
mod tests {
    use crate::utils::captcha::email::send_email;
    use crate::utils::captcha::Captcha;
    use crate::utils::captcha::Usage;
    #[test]
    fn test_send_email_ok() {
        let code = Captcha::new(
            "eddy1guo@gmail.com".to_string(),
            "1".to_string(),
            Usage::Register,
        );
        let res = send_email("123456", &code.owner).unwrap();
        println!("res {:?}", res);
    }
}
