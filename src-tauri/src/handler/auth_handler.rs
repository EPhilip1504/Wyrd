use anyhow::Error;
use axum::{
    http::{response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_macros::debug_handler;
use fast_chemail;
use hex_literal::hex;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use rustls::client;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use sqlx::{PgPool, Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use totp_rs::Secret;
use tracing::{debug, error, instrument};
use tracing_subscriber::field::display;

use crate::{
    auth_service::{login, signup, AuthenticationErrors},
    otp::{generate_otp, send_otp, verify_otp},
    password::encrypt,
};

#[derive(Deserialize)]
pub struct SignupReq {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct OTPVerReq {
    pub entered_code: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    //The user can either log in with a username or email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Theme {
    pub mode: String,
    pub rbg: (u8, u8, u8),
}

/*#[derive(Deserialize)]
pub struct Notifications {

}*/
pub type Db = Pool<Postgres>;

pub struct AppState {
    pub db: Db,
    pub red: MultiplexedConnection,
}

const EXPIRY_TIME: u64 = 90;

pub async fn signup_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<SignupReq>,
) -> impl IntoResponse {
    let pool = &state.db;

    let signup_response = signup(pool, &payload).await;

    let ref_client_email = payload.email.as_str();
    let ref_client_name = payload.name.as_str();

    match signup_response {
        Ok(_) => {
            let mut connection = state.red.clone();
            let (_, _): ((), ()) = redis::pipe()
                .set("client_email_signup", ref_client_email)
                .set("client_name_signup", ref_client_name)
                .query_async(&mut connection)
                .await
                .unwrap();
            let token = match generate_otp(&pool, ref_client_email, ref_client_name).await {
                Ok(t) => t.generate_current().unwrap(),
                Err(err) => {
                    error!("Failed to generate OTP: {:?}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to generate OTP"})),
                    );
                }
            };

            let shared_token = token.as_str();

            let encrypted_token = encrypt(shared_token).await.unwrap();

            let _: () = redis::pipe()
                .atomic()
                .set_ex("token", encrypted_token, EXPIRY_TIME)
                .ignore()
                .query_async(&mut connection)
                .await
                .unwrap();

            send_otp(shared_token, ref_client_email, ref_client_name).await;

            (
                StatusCode::OK,
                Json(json!({"message": "Signup successful"})),
            )
        }
        Err(errors) => {
            // Iterate over the Vec<AuthenticationErrors> and return the first matching error
            for error in errors {
                match error {
                    AuthenticationErrors::SignupErrorUsername(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": format!("Username: {:?}", error)})),
                        );
                    }
                    AuthenticationErrors::SignupErrorEmail(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": format!("Email: {:?}", error)})),
                        );
                    }

                    AuthenticationErrors::DatabaseError(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": format!("DatabaseError: {:?}", error)})),
                        );
                    }

                    AuthenticationErrors::HashError(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": format!("HashingError: {:?}", error)})),
                        );
                    }
                    _ => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "An unexpected error occurred."})),
                        );
                    }
                }
            }

            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Unhandled error occurred."})),
            )
        }
    }
}
//        let token = email_verification::generate_otp(&pool, payload.email, payload.name);

pub async fn otp_verify_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<OTPVerReq>,
) -> impl IntoResponse {
    let pool = &state.db;

    let mut connection = state.red.clone();
    let (client_email, client_name): (String, String) = redis::pipe()
        .get("client_email_signup".to_string())
        .get("client_name_signup".to_string())
        .query_async(&mut connection)
        .await
        .unwrap();

    let entered_code = payload.entered_code.as_str();
    let mut con = state.red.clone();

    let mut exists = con.exists::<&str, i64>("token").await.unwrap();
    if exists == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Your one-time password has expired. Please resend the otp, then check your inbox and enter the new code to continue."
            })),
        );
    }

    let code: String = redis::cmd("GET")
        .arg("token")
        .query_async(&mut con)
        .await
        .unwrap();

    let email_verification_response = verify_otp(Extension(state), entered_code).await;
    let encrypted_code = encrypt(entered_code).await.unwrap();

    match email_verification_response {
        Ok(true) => {
            connection.del::<&str, i64>("token").await.unwrap();
            connection
                .del::<&str, String>("client_email_signup")
                .await
                .unwrap();
            sqlx::query!(
                "UPDATE users SET user_verified = true WHERE username = $1",
                client_name,
            );
            return (
                StatusCode::OK,
                Json(json!({"Valid": "The code entered is valid."})),
            );
        }
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"Invalid": "The code entered is invalid."})),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!(e.to_string()))),
    }
}

//Personalized handlers just store the info in the Database.

pub async fn personalize_theme(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<Theme>,
) -> Result<impl IntoResponse, anyhow::Error> {
    let pool = &state.db;
    let mut con = state.red.clone();
    let client_name = con.get::<&str, String>("client_name_signup").await.unwrap();
    sqlx::query!(
        "UPDATE users SET personalization = personalization || $1::jsonb WHERE name = $2",
        serde_json::json!({"theme": payload.mode,"color":payload.rbg}),
        client_name
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthenticationErrors::DatabaseError(e))?;
    Ok(StatusCode::OK)
}

pub async fn personalize_notification(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<OTPVerReq>,
) -> impl IntoResponse {
}

pub async fn personalize_accesibility(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<OTPVerReq>,
) -> impl IntoResponse {
}

pub async fn personalize_avatar(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<OTPVerReq>,
) -> impl IntoResponse {
}

pub async fn login_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<LoginReq>,
) -> impl IntoResponse {
    let pool = &state.db;
    match login(&pool, payload).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Login successful"}))),
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": format!("Login failed: {}", err)})),
        ),
    }
}
