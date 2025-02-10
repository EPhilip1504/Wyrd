use axum::http::header::FROM;

use crate::auth_service::AuthenticationErrors;
use lettre::message::header::ContentType;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use sqlx::{
    types::{time::PrimitiveDateTime, Json},
    PgPool,
};
use std::time::{SystemTime, SystemTimeError};
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::{debug, error, instrument};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

const FULL_COMPANY_EMAIL: &str = "Wyrd <thewyrdteam@gmail.com>";
const COMPANY_EMAIL: &str = "thewyrdteam@gmail.com";

pub async fn generate_otp(
    pool: &PgPool,
    client_email: String,
    client_name: String,
) -> Result<TOTP, AuthenticationErrors> {
    debug!("generate_otp called with client_email: {client_email}, client_name: {client_name}");

    let sk = sqlx::query!(
        "SELECT totp_secret FROM users WHERE email = $1",
        client_email
    )
    .fetch_optional(pool) // Fetch an optional record from the database
    .await
    .map_err(|e| AuthenticationErrors::DatabaseError(e))? // Map database errors
    .unwrap()
    .totp_secret // Access the totp_secret field
    .unwrap();

    let totp = TOTP::new(
        Algorithm::SHA512,
        6,
        1,
        60,
        Secret::Raw(sk.as_bytes().to_vec()).to_bytes().unwrap(),
        Some("Wyrd".to_string()),
        client_email,
    )
    .unwrap();

    let token = totp.generate_current().unwrap();

    debug!("Generated OTP token: {token}");

    Ok((totp))
}

pub async fn send_otp(
    token: String,
    client_email: String,
    client_name: String,
) -> Result<(), AuthenticationErrors> {
    let full_client_email: String = format!("{} <{}>", client_name, client_email);

    let send_email = Message::builder()
        .from(FULL_COMPANY_EMAIL.parse().unwrap())
        .reply_to(FULL_COMPANY_EMAIL.parse().unwrap())
        .to(full_client_email.parse().unwrap())
        .subject("Please verify your account")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from(format!(
            "Welcome {}! Thank you for joining Wyrd! Please enter the following code shown: {}",
            client_name, token,
        )))
        .unwrap();

    let gmail_creds = Credentials::new(
        "thewyrdteam@gmail.com".to_owned(),
        "xgzx svcc jfoe woxs ".to_owned(),
    );

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(gmail_creds)
        .build();

    match mailer.send(&send_email) {
        Ok(_) => {
            debug!("Email sent successfully to {client_email}");
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email: {e:?}");
            Err(AuthenticationErrors::EmailSendError(format!(
                "Failed to send email: {e:?}"
            )))
        }
    }
}

pub async fn verify_otp(
    pool: &PgPool,
    generated_code: TOTP,
    entered_code: String,
) -> Result<(bool), SystemTimeError> {
    match generated_code.check_current(entered_code.as_str()) {
        Ok(b) => return Ok(b),
        Err(st) => return Err(st),
    }
}
