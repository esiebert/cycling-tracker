use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tonic::{Code, Request};

use crate::common::run_test_env;
use cycling_tracker::cycling_tracker::{Credentials, SignUpResult};

lazy_static! {
    static ref CREDENTIALS: Credentials = Credentials {
        username: "User".to_string(),
        password: "Password".to_string(),
    };
}

#[sqlx::test]
async fn test_new_user_login(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let response = test_env
        .auth_service
        .sign_up(Request::new((*CREDENTIALS).clone()))
        .await
        .expect("Failed to sign up")
        .into_inner();

    assert_eq!(response, SignUpResult { result: true });

    test_env
        .auth_service
        .login(Request::new((*CREDENTIALS).clone()))
        .await
        .expect("Failed to login");
}

#[sqlx::test]
async fn test_user_already_exists(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let response = test_env
        .auth_service
        .sign_up(Request::new((*CREDENTIALS).clone()))
        .await
        .expect("Failed to sign up")
        .into_inner();

    assert_eq!(response, SignUpResult { result: true });

    let response = test_env
        .auth_service
        .sign_up(Request::new((*CREDENTIALS).clone()))
        .await
        .expect_err("User created when it shouldn't have");

    assert_eq!(response.code(), Code::AlreadyExists);
}

#[sqlx::test]
async fn test_invalid_credentials(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let response = test_env
        .auth_service
        .login(Request::new((*CREDENTIALS).clone()))
        .await
        .expect_err("Login succeeded when it shouldn't have");

    assert_eq!(response.code(), Code::Unauthenticated);
}
