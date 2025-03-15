use crate::auth_handler::AppState;
use crate::auth_service::AuthenticationErrors;
use anyhow::anyhow;
use axum::extract::FromRef;
use tauri::ipc::Invoke;
use thiserror::Error;

use crate::password::encrypt;
use axum::http::header::FROM;
use axum::Extension;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lettre::message::header::ContentType;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use redis::AsyncCommands;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::{
    types::{time::PrimitiveDateTime, Json},
    PgPool,
};
use std::sync::Arc;
use std::time::{SystemTime, SystemTimeError};
use tauri::{State, StateManager};
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::{debug, error, instrument};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

const FULL_COMPANY_EMAIL: &str = "Wyrd <thewyrdteam@gmail.com>";
const COMPANY_EMAIL: &str = "thewyrdteam@gmail.com";

#[derive(Debug)]
pub struct OTPInfo {
    pub name: String,
    pub email: String,
}

impl serde::Serialize for OTPInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("OTPInfo", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("email", &self.email)?;
        state.end()
    }
}

#[derive(Debug, Deserialize, Serialize, Error)]
pub enum OTPErrors {
    #[error("Sending OTP Failed: {0}")]
    SendOTPError(String),
    #[error("Resending OTP Failed: {0}")]
    ResendOTPError(String),
    #[error("Sending Email Failed: {0}")]
    EmailError(String),
    #[error("Generating OTP Failed: {0}")]
    GenerateOTPError(String),
}

//impl Serialize for OTPErrors {}

impl IntoResponse for OTPErrors {
    fn into_response(self) -> axum::response::Response {
        let body = match self {
            OTPErrors::SendOTPError(..) => "Error Sending OTP",
            OTPErrors::ResendOTPError(..) => "Error Resending OTP",
            OTPErrors::EmailError(..) => "Error sending email",
            OTPErrors::GenerateOTPError(..) => "Error generating OTP",
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

pub async fn generate_otp(
    pool: &PgPool,
    client_email: &str,
    client_name: &str,
) -> Result<TOTP, anyhow::Error> {
    let sk = sqlx::query!(
        "SELECT totp_secret FROM users WHERE email = $1",
        client_email
    )
    .fetch_optional(pool) // Fetch an optional record from the database
    .await
    .map_err(|e| (e))? // Map database errors
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
        client_email.to_string(),
    )
    .unwrap();

    let token = totp.generate_current().unwrap();

    debug!("Generated OTP token: {token}");

    Ok((totp))
}

pub async fn send_otp(token: &str, client_email: &str, client_name: &str) -> Result<(), OTPErrors> {
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
            Err(OTPErrors::EmailError(format!(
                "Failed to send email: {e:?}"
            )))
        }
    }
}

/*

*/

pub async fn verify_otp(
    Extension(state): Extension<Arc<AppState>>,
    entered_code: &str,
) -> Result<(bool), anyhow::Error> {
    let mut con = state.red.clone();

    //Check if otp exists (already expired)
    let mut exists = con.exists::<&str, i64>("token").await?;
    match exists {
        0 => {
            return Err(anyhow!("Your one-time password has expired. Please resend the otp, then check your inbox and enter the new code to continue."));
        }
        1 => {
            // If the code does exist (not expired) check if it's valid
            let code: String = redis::cmd("GET").arg("token").query_async(&mut con).await?;
            let encrypted_code = encrypt(entered_code).await?;
            return Ok(encrypted_code.trim() == code.trim());
        }
        _ => return Err(anyhow!("An error has occured with verifying the code.")),
    }
}
