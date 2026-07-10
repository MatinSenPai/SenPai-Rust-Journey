//! Drives the whole stack — router, `require_auth` middleware, protected
//! handler — through `tower::ServiceExt::oneshot`, the same technique as
//! `taskforge-api/tests/api_test.rs` and the anime catalog lesson's
//! `tests/api_test.rs`. Only the crate's public surface (`app`,
//! `issue_token`, `Claims`) is used here — exactly like a real consumer of
//! this crate would use it.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;

use p3_06_02_jwt_and_tower_middleware::{app, issue_token, Claims};

const SECRET: &str = "test-secret-do-not-use-in-prod";

/// Mints a token whose `exp` is already in the past — `require_auth`
/// should reject this the same way it rejects a missing or malformed
/// token, since `jsonwebtoken::decode` checks `exp` against the current
/// time automatically.
fn expired_token() -> String {
    let one_hour_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .saturating_sub(3600);
    let claims = Claims {
        sub: "user-1".to_string(),
        exp: one_hour_ago as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn request_with_auth_header(value: Option<String>) -> Request<Body> {
    let mut builder = Request::builder().uri("/whoami");
    if let Some(value) = value {
        builder = builder.header(header::AUTHORIZATION, value);
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn valid_token_reaches_the_protected_handler() {
    let token = issue_token("user-42", SECRET);

    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(Some(format!("Bearer {token}"))))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["user_id"], "user-42");
}

#[tokio::test]
async fn missing_authorization_header_returns_401() {
    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn header_without_bearer_prefix_returns_401() {
    let token = issue_token("user-42", SECRET);

    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(Some(token))) // no "Bearer " prefix
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn garbage_token_returns_401() {
    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(Some(
            "Bearer not-a-real-jwt".to_string(),
        )))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_signed_with_a_different_secret_returns_401() {
    let token = issue_token("user-42", "a-completely-different-secret");

    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(Some(format!("Bearer {token}"))))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_token_returns_401() {
    let response = app(SECRET.to_string())
        .oneshot(request_with_auth_header(Some(format!(
            "Bearer {}",
            expired_token()
        ))))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
