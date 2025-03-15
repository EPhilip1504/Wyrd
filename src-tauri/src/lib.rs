// lib.rs
#![allow(unused_imports)]
#![allow(unused)]
#![allow(warnings)]

#[path = "handler/auth_handler.rs"]
pub mod auth_handler;

#[path = "service/auth_service.rs"]
pub mod auth_service;

#[path = "service/otp.rs"]
pub mod otp;

#[path = "utils/password.rs"]
pub mod password;

//use auth_handler::{otp_verify_handler, resend_otp_handler, signup_handler, AppState};
