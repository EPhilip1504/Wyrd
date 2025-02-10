#![allow(unused_imports)]
#![allow(unused)]
#![allow(warnings)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::{Context, Result};
use auth_handler::SignupReq;
use axum::{
    http::{self, Method},
    response::{Html, IntoResponse},
    routing::{get, post},
    BoxError, Extension, Router,
};
use dotenvy::dotenv;
use http::HeaderValue;
use log::info;
use redis::{self, aio::MultiplexedConnection, AsyncCommands};
use serde_json::Value;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};
use sqlx::{migrate::MigrateDatabase, postgres::PgPoolOptions, Pool, Postgres};
use std::{env, error::Error, fmt::format, sync::Arc, thread};
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{self, Listener, State};
use tauri::{async_runtime::block_on, Emitter};
use tauri::{generate_context, generate_handler, AppHandle, Builder, Manager, Wry};
use tauri_plugin_autostart::MacosLauncher;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, Session, SessionManagerLayer,
};
use tower_sessions_sqlx_store::{sqlx::PgPool, PostgresStore};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{self, layer};
#[path = "service/auth_service.rs"]
mod auth_service;

#[path = "handler/auth_handler.rs"]
mod auth_handler;

#[path = "service/email_verification.rs"]
pub mod email_verification;

type Db = Arc<Pool<Postgres>>;

struct AppState {
    db: Db,
    red: MultiplexedConnection,
}

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

    Ok(Arc::new(db))
}

async fn setup_reddis() -> Result<(MultiplexedConnection), anyhow::Error> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_multiplexed_tokio_connection().await?;
    Ok((con))
}

async fn setup_app(app: &AppHandle, state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "tauri://localhost".parse::<HeaderValue>().unwrap(),
        ]) //
        .allow_methods([Method::GET, Method::POST]) //
        .allow_credentials(true)
        .allow_headers([
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ]);
    /* */

    let start_app = Router::new()
        .route("/signup", post(auth_handler::signup_handler))
        .route("/otp", post(auth_handler::otp_verify_handler))
        .route("/resend-otp", post(auth_handler::resend_otp_handler))
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
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
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
