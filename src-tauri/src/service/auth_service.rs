use anyhow::Error;
use axum::Router;
use chrono::{DateTime, Local, Utc};
use email_verification::{generate_otp, send_otp};
use sqlx::{
    error,
    types::{time::PrimitiveDateTime, Json},
    PgPool,
};
use std::{fmt::Display, path, ptr::null, vec};
use thiserror::Error;
use totp_rs::Secret;

#[path = "../service/email_verification.rs"]
mod email_verification;

#[path = "../utils/password.rs"]
mod password;

use crate::auth_handler::{LoginReq, OTPReq, SignupReq};

#[derive(Debug, Error)]
pub enum AuthenticationErrors {
    #[error("Password hashing failed: {0}")]
    HashError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Incorrect Username or Password {0}")]
    LoginError(String),

    #[error("Username already taken. Please try a different one.")]
    SignupErrorUsername(String),

    #[error("This email address has already been regestred with a different account. Please try a different one or log in.")]
    SignupErrorEmail(String),

    #[error("Incorrect code")]
    InvalidOTP(String),

    #[error("An error has ocurred. Please try again later.")]
    GeneralError(String),

    #[error("")]
    EmailSendError(String),
}

pub async fn signup(pool: &PgPool, payload: &SignupReq) -> Result<(), Vec<AuthenticationErrors>> {
    let mut errors: Vec<AuthenticationErrors> = Vec::new();
    /*match fetch_email_data(&payload.email) {
        Ok(response) => println!("{:?}", response),
        Err(e) => errors.push(AuthenticationErrors::SignupInvalidEmail(e.to_string())),
    }*/

    let existing_username = sqlx::query!(
        "SELECT id FROM users WHERE username = $1",
        &payload.username,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| vec![AuthenticationErrors::DatabaseError(e)])?;

    let existing_email = sqlx::query!("SELECT id FROM users WHERE email = $1", &payload.email,)
        .fetch_optional(pool)
        .await
        .map_err(|e| vec![AuthenticationErrors::DatabaseError(e)])?;

    if existing_username.is_some() {
        errors.push(AuthenticationErrors::SignupErrorUsername(
            ("Username already taken. Please try a different one.".to_string()),
        ));
    }
    if existing_email.is_some() {
        errors.push(AuthenticationErrors::SignupErrorEmail(("This email address has already been regestred with a different account. Please try a different one or log in.".to_string())));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    // Hash the password
    let hashed_password = password::hash(payload.password.clone())
        .await
        .map_err(|e| vec![AuthenticationErrors::HashError(e.to_string())])?;

    let secret_key = Secret::generate_secret().to_string();

    sqlx::query!(
          "INSERT INTO users (name, username, email, password, last_login, status, activity, user_verified, totp_secret)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
          &payload.name,                  // $1: User's name
          &payload.username,              // $2: User's username
          &payload.email,                 // $3: User's email
          hashed_password,               // $4: Hashed password
          PrimitiveDateTime::MAX, // $5: last_login (None for new users)
          "active",                      // $6: status (default to "active")
          "offline",                     // $7: activity (default to "offline")
          false,                          // $8: user_verified (default to false)
          secret_key,
      )
    .execute(pool)
    .await
    .map_err(|e| vec![AuthenticationErrors::DatabaseError(e)])?;

    Ok(())
}

pub async fn login(pool: &PgPool, payload: LoginReq) -> Result<(), AuthenticationErrors> {
    let user = sqlx::query!(
        "SELECT password FROM users WHERE username = $1 OR email = $2",
        payload.username,
        payload.email
    )
    .fetch_optional(pool)
    .await?;

    match user {
        Some(record) => {
            if password::verify(&pool, payload.password, record.password)
                .await
                .unwrap_or(false)
            {
                Ok(())
            } else {
                Err(AuthenticationErrors::LoginError(
                    "Your username or password is incorrect.".to_string(),
                ))
            }
        }
        None => Err(AuthenticationErrors::LoginError(
            "User not found".to_string(),
        )),
    }
}
