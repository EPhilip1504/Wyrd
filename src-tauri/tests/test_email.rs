#![allow(unused)]
#![allow(warnings)]
#[cfg(test)]
use axum::http::header::FROM;
use chrono::prelude::*;
use hex_literal::hex;
use lettre::message::header::ContentType;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use redis::{Commands, ToRedisArgs};
use sha2::{Digest, Sha256, Sha512};
use sqlx::{
    types::{time::PrimitiveDateTime, Json},
    PgPool,
};
use std::hash::Hasher;
use std::thread;
use std::time::{Duration, SystemTime};
use text_io::read;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing_subscriber::fmt::format;

pub async fn encrypt(code: &str) -> Result<(String), anyhow::Error> {
    let mut hasher = Sha512::new();
    hasher.update(code.as_bytes());
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

#[tokio::test]
pub async fn generate_otp() -> Result<(), anyhow::Error> {
    let client_email: &str = "elijahphilip0706@gmail.com";
    let client_name: &str = "Elijah Philip";
    const COMPANY_EMAIL: &str = "thewyrdteam@gmail.com";
    let secret_key = Secret::generate_secret().to_string();

    let full_client_email: String = format!("{} <{}>", client_name, client_email);

    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    const FULL_COMPANY_EMAIL: &str = "Wyrd <thewyrdteam@gmail.com>";
    let totp = TOTP::new(
        Algorithm::SHA512,
        6,
        1,
        90,
        Secret::Raw(secret_key.as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
        Some("Wyrd".to_string()),
        client_email.to_string(),
    )
    .unwrap();

    let timestamp = Local::now().timestamp();
    let datetime: DateTime<Local> = Local.timestamp_opt(timestamp, 0).unwrap();
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");

    let code = totp.generate_current()?;
    println!("{}", code);
    let encoded_hashed_sha512 = encrypt(code.as_str()).await?;

    println!("SHA-512: {:?}", encoded_hashed_sha512);

    let _: () = redis::pipe()
        .atomic()
        .set_ex("code", encoded_hashed_sha512.clone(), totp.step)
        .ignore()
        .query(&mut con)?;

    let input_code: String = read!();
    let entered_code = encrypt(input_code.as_str()).await?;

    println!("{}", entered_code == encoded_hashed_sha512);
    // println!(
    //     "code {}\n ttl {}\n valid until: {}\n timestamp {}\n",
    //     totp.generate(prev_timestamp as u64),
    //     totp.ttl()?,
    //     newdate,
    //     timestamp,
    // );

    //println!("previous timestamp: {:?} AKA {}\n", prev_timestamp, nd2,);
    //print!("Is {} still valid? {}", totp)
    //println!("{:?}", totp.next_step(time));

    let mut exists = con.exists::<&str, i64>("code")?;
    match exists {
        0 => {
            println!("The key does not exist")
        }
        1 => {
            let code: String = redis::cmd("GET").arg("code").query(&mut con)?;
            println!("The key exists. Code: {}\n", code);
        }
        _ => {
            println!("An Error has occurred")
        }
    }
    /*
    while exists > 0 {
        exists = con.exists::<&str, i64>("code")?;
        let f = con.ttl::<&str, i64>("timestamp")?;
        println!("{}", f);
        thread::sleep(Duration::from_secs(1));
        thread::scope(|s| {
            s.spawn(|| {
                let timestamp = con.get::<&str, i64>("timestamp").unwrap();
                let code = totp.generate(timestamp as u64);
                println!("{}", code);
            })
            .join()
            .unwrap()
        })
    }*/
    Ok(())
}
