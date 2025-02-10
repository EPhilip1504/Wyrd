#![allow(unused)]
#![allow(warnings)]
#[cfg(test)]
#[path = "../src/service/auth_service.rs"]
mod auth_service;

#[path = "../src/handler/auth_handler.rs"]
mod auth_handler;

use std::sync::Arc;

use axum::{
    http::{response, Response},
    Extension,
};
use sqlx::PgPool;

mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_check_users() {
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Failed to connect to the database");

        // Provide a test username
        let username = "john".to_string();
        let email = "john@example.com";

        let users = sqlx::query!("SELECT totp_secret FROM users WHERE email = $1", email)
            .fetch_optional(&pool)
            .await
            .expect("REASON");

        println!("{:?}", users.is_some());
        println!("{:?}", users);
    }
}
/*
async fn test_signup(){
  let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");


}

#[tokio::test]
async fn test_signup_handler() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");

    let payload1 = auth_handler::SignupReq {
        name: "John Doe".into(),
        username: "johndoe".into(),
        email: "john@example.com".into(),
        password: "securepassword".into(),
    };

    let payload2 = auth_handler::SignupReq {
      name: "Mark Doe".into(),
      username: "Markdoe".into(),
      email: "Mark@example.com".into(),
      password: "securepassword".into(),
    };

    let payload3 = auth_handler::SignupReq{
      name:"Ryanh Quinn".into(),
      username: "stk".into(),
      email: "ryan@example.com".into(),
      password: "securepassword".into(),
    };

    let payload4 = auth_handler::SignupReq{
      name:"Ryanh Quinn".into(),
      username: "stk".into(),
      email: "ryan4@example.com".into(),
      password: "securepassword".into(),
    };

    let response1 = auth_handler::signup_handler(
        Extension(Arc::new(pool.clone())),
        axum::Json(payload1),
    )
    .await;

    let response2 = auth_handler::signup_handler(
      Extension(Arc::new(pool.clone())),
      axum::Json(payload2),
  )
  .await;

  let response3 = auth_handler::signup_handler(Extension(Arc::new(pool.clone())),axum::Json(payload3),).await;

  let response4 = auth_handler::signup_handler(Extension(Arc::new(pool.clone())),axum::Json(payload4),).await;




  let users = sqlx::query!("SELECT id, username, email, password FROM users")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch users");

        for user in users {
            println!("User ID: {}, Username: {}, Email: {}, Password: {}", user.id, user.username, user.email, user.password);
        }

}




#[tokio::test]
async fn test_login_handler(){
  let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");


}
*/
