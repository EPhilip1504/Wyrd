// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(unused)]
#![allow(warnings)]
use anyhow::{Context, Result};
use axum::{
    debug_handler,
    http::{self, Method, Response, StatusCode},
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post},
    BoxError, Extension, Router,
};
use core::error;
use dotenvy::dotenv;
use http::HeaderValue;
use log::info;
use redis::Commands;
use redis::{self, aio::MultiplexedConnection, AsyncCommands};
use serde_json::Value;
use sha2::digest::DynDigest;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use std::{env, error::Error, fmt::format, sync::Arc, thread};
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{self, Listener, State};
use tauri::{async_runtime::block_on, Emitter};
use tauri::{generate_context, generate_handler, AppHandle, Builder, Manager, Wry};
use tauri_plugin_autostart::MacosLauncher;
use thiserror::Error;
use tower::ServiceBuilder;
use tower_http::{cors::Any, services::ServeDir};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, Session, SessionManagerLayer,
};
use tower_sessions_sqlx_store::{sqlx::PgPool, PostgresStore};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{self, layer};
use wyrd_lib::password::encrypt;

use wyrd_lib::auth_service::AuthenticationErrors;
use wyrd_lib::otp::{generate_otp, send_otp, verify_otp, OTPErrors, OTPInfo};
use wyrd_lib::{
    auth_handler::{login_handler, otp_verify_handler, signup_handler, AppState, Db},
    auth_service,
};

const EXPIRY_TIME: u64 = 90;

async fn setup_db() -> Result<(Db), anyhow::Error> {
    dotenv().ok();
    let database_url = dotenvy::var("DATABASE_URL")
        .context("DATABASE_URL must be set")
        .unwrap();

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    Ok(db)
}

async fn setup_reddis() -> Result<(MultiplexedConnection), anyhow::Error> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_multiplexed_tokio_connection().await?;
    Ok((con))
}

async fn setup_app(app: &AppHandle, state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let cors_layer = CorsLayer::new()
        .allow_origin(
            Any, /*[
                    "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                    "tauri://localhost".parse::<HeaderValue>().unwrap(),
                ]*/
        ) //
        .allow_methods(Any /*[Method::GET, Method::POST]*/) //
        .allow_credentials(false)
        .allow_headers(
            Any, /*[
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                ]*/
        );
    /* */

    let start_app = Router::new()
        .route("/signup", post(signup_handler))
        .route("/otp", post(otp_verify_handler))
        //.route("personalize-1", post())
        //.route("personalize-2", post())
        //.route("personalize-3", post(func3))
        //.route("personalize-4", post(func4))
        //.route("/personalize", method_router)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(Extension(state))
                .layer(cors_layer),
        )
        // Add explicit separation between API and frontend
        .nest_service("/app", ServeDir::new("dist"));

    tauri::async_runtime::spawn(async {
        axum::serve(listener, start_app)
            .await
            .expect("Server failed");
    });

    Ok(())
}

/*
fn verify_local_process()  {
    let s = System::new_all();
    for process in s.processes_by_exact_name("backend".as_ref()) {
        if process.
    }
}*/

#[tauri::command]
async fn client_info_otp(state: State<'_, Arc<AppState>>) -> Result<OTPInfo, ()> {
    let mut connection = state.red.clone();

    let ((client_email, client_name)): (String, String) = redis::pipe()
        .get("client_email_signup".to_string())
        .get("client_name_signup".to_string())
        .query_async(&mut connection)
        .await
        .unwrap();

    let otp_info = OTPInfo {
        name: client_name,
        email: client_email,
    };

    Ok(otp_info)
}

#[tauri::command]
async fn resend_otp_handler(state: State<'_, Arc<AppState>>) -> Result<(), OTPErrors> {
    let pool = &state.db;
    let mut connection = state.red.clone();
    let code_exists = connection.exists::<&str, i64>("token").await.unwrap();
    if code_exists == 1 {
        connection.del::<&str, i64>("token").await.unwrap();
    }
    let ((client_email, client_name)): (String, String) = redis::pipe()
        .get("client_email_signup".to_string())
        .get("client_name_signup".to_string())
        .query_async(&mut connection)
        .await
        .unwrap();

    let ref_client_email = client_email.as_str();
    let ref_client_name = client_name.as_str();

    let token = match generate_otp(&pool, ref_client_email, ref_client_name).await {
        Ok(t) => t.generate_current().unwrap(),
        Err(err) => return Err(OTPErrors::GenerateOTPError(err.to_string())),
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

    send_otp(shared_token, ref_client_email, ref_client_name).await?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            client_info_otp,
            resend_otp_handler
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on({
                let handle = handle;
                async move {
                    let db = setup_db().await?;
                    let red = setup_reddis().await?;
                    let state = Arc::new(AppState { db, red });
                    app.manage(state.clone());
                    setup_app(&handle, state).await
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
