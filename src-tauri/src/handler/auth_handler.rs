use auth_service::AuthenticationErrors;
use axum::{
    http::{response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use redis::AsyncCommands;
use rustls::client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tauri::State;
use tauri::{AppHandle, Emitter};
use totp_rs::Secret;
use tracing::{debug, error, instrument};

use crate::AppState;

#[path = "../service/email_verification.rs"]
mod email_verification;

#[path = "../service/auth_service.rs"]
mod auth_service;

#[path = "../utils/password.rs"]
mod password;

#[derive(Deserialize)]
pub struct SignupReq {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct OTPVerReq {
    pub entered_code: OTPReq,
    pub ttl: u64,
    pub signup: SignupReq,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String, //This can be either an email or username
    pub username: String,
    pub password: String,
}
pub async fn signup_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<SignupReq>,
) -> impl IntoResponse {
    let pool = &state.db;

    let signup_response = auth_service::signup(&pool, &payload).await;

    match signup_response {
        Ok(_) => {
            let mut connection = state.red.clone();
            let ((_, _)): ((), ()) = redis::pipe()
                .set("client_email_signup", &payload.email)
                .set("client_name_signup", &payload.name)
                .query_async(&mut connection)
                .await
                .unwrap();
            let token = match email_verification::generate_otp(
                &pool,
                payload.email.clone(),
                payload.name.clone(),
            )
            .await
            {
                Ok(t) => t.generate_current().unwrap(),
                Err(err) => {
                    error!("Failed to generate OTP: {:?}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to generate OTP"})),
                    );
                }
            };

            email_verification::send_otp(token, payload.email, payload.name).await;

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
    let ((client_email, client_name)): (String, String) = redis::pipe()
        .get_del("client_email_signup".to_string())
        .get_del("client_name_signup".to_string())
        .query_async(&mut connection)
        .await
        .unwrap();

    let token = match email_verification::generate_otp(&pool, client_email, client_name).await {
        Ok(t) => t,
        Err(err) => {
            error!("Failed to generate OTP: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Fa
                        iled to generate OTP"})),
            );
        }
    };

    let email_verification_response =
        email_verification::verify_otp(&pool, token, payload.entered_code).await;

    match email_verification_response {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"Correct": "The code entered is correct."})),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"Incorrect": "The code entered is incorrect."})),
        ),
    }
}

pub async fn resend_otp_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<SignupReq>,
) -> impl IntoResponse {
    let pool = &state.db;
    let token =
        match email_verification::generate_otp(&pool, payload.email.clone(), payload.name.clone())
            .await
        {
            Ok(t) => t.generate_current().unwrap(),
            Err(err) => {
                error!("Failed to generate OTP: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to generate OTP"})),
                );
            }
        };

    match email_verification::send_otp(token, payload.email, payload.name).await {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(json!({"OTP SENT": "OTP SUCESSFULLY SENT TO EMAIL"})),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to generate OTP"})),
            );
        }
    }
}

pub async fn login_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<LoginReq>,
) -> impl IntoResponse {
    let pool = &state.db;
    match auth_service::login(&pool, payload).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Login successful"}))),
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": format!("Login failed: {}", err)})),
        ),
    }
}
