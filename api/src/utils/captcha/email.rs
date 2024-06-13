use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use anyhow::Result;
use common::env::CONF;

use lettre::transport::smtp::client::{Tls, TlsParameters};
use tracing::debug;

pub fn send_email(to_mail: &str, content: &str) -> Result<()> {
    let from = CONF.stmp.sender.parse::<Mailbox>()?;
    let to = to_mail.parse::<Mailbox>()?;

    // create email content
    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("[ChainLess] Captcha")
        .header(ContentType::TEXT_PLAIN)
        .body(content.to_owned())?;

    let creds = Credentials::new(CONF.stmp.sender.clone(), CONF.stmp.password.clone());

    let tls = TlsParameters::builder(CONF.stmp.server.clone())
        .dangerous_accept_invalid_certs(true)
        .build()?;

    let mailer = SmtpTransport::relay(CONF.stmp.server.as_str())
        .map(|c| c.port(1025))?
        .tls(Tls::Required(tls))
        .credentials(creds)
        .build();

    let send_res = mailer.send(&email)?;
    debug!("mail send res {:?}", send_res);
    Ok(())
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
