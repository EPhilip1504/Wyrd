#![allow(unused)]
#![allow(warnings)]
#[cfg(test)]
use axum::http::header::FROM;
use lettre::message::header::ContentType;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use sqlx::{
    types::{time::PrimitiveDateTime, Json},
    PgPool,
};
use std::time::SystemTime;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing_subscriber::fmt::format;

#[tokio::test]
pub async fn generate_otp() {
    let client_email: &str = "elijahphilip0706@gmail.com";
    let client_name: &str = "Elijah Philip";
    const COMPANY_EMAIL: &str = "thewyrdteam@gmail.com";
    let secret_key = Secret::generate_secret().to_string();

    let full_client_email: String = format!("{} <{}>", client_name, client_email);

    const FULL_COMPANY_EMAIL: &str = "Wyrd <thewyrdteam@gmail.com>";
    let totp = TOTP::new(
        Algorithm::SHA512,
        6,
        1,
        60,
        Secret::Raw(secret_key.as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
    )
    .unwrap();

    println!(
        "code {}\t ttl {}\t valid until: {}",
        totp.generate_current().unwrap(),
        totp.ttl().unwrap(),
        totp.next_step_current().unwrap()
    );
    //println!("{:?}", totp.next_step(time));
}
