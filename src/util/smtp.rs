/**
 * SMTP client setup, for sending notification & password reset emails.
 */

use std::error::Error;
use std::fs;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{ Message, SmtpTransport, Transport };
use serde::Deserialize;
use tokio::sync::OnceCell;

pub static MAILER: OnceCell<SmtpTransport> = OnceCell::const_new();

#[derive(Deserialize)]
struct SecretsConfig {
    smtp: SecretsConfigSmtp,
}

#[derive(Deserialize)]
struct SecretsConfigSmtp {
    username: String,
    password: String,
    relay_server_name: String,
}

pub fn init_mailer() {
    let secrets_toml = fs::read_to_string(format!(
        "{}/config/secrets.toml",
        env!("CARGO_MANIFEST_DIR")
    )).expect("Failed to read secrets.toml file.");
    let config: SecretsConfig = toml::from_str(&secrets_toml)
        .expect("Failed to parse secrets.toml file.");
    
    tracing::info!("SMTP username: {}", config.smtp.username);
    tracing::info!("SMTP relay server name: {}", config.smtp.relay_server_name);

    let credentials = Credentials::new(config.smtp.username.to_owned(), config.smtp.password.to_owned());

    let mailer = SmtpTransport::starttls_relay(&config.smtp.relay_server_name)
        .unwrap()
        .credentials(credentials)
        .build();
    
    MAILER.set(mailer).expect("Mailer already initialized.");
}

pub fn send_email(to: &str, subject: String, body: String, content_type: ContentType) -> Result<(), Box<dyn Error + Send + Sync>> {
    let email = Message::builder()
        .from("SupercarBand Notification <noreply@supercarband.com>".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(content_type)
        .body(body)
        .unwrap();
    let mailer = MAILER.get().expect("Mailer is not initialized.");
    match mailer.send(&email) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            Err(Box::new(e))
        },
    }
}
