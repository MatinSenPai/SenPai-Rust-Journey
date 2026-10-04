//! Tests for the Build rung: `ValidatedJson<T>` works for any `T`, not just
//! `NewReview`, and keeps `Json<T>`'s own rejections.

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use tower::ServiceExt;
use validator::Validate;

use p3_03_02_validation::ValidatedJson;

#[derive(Deserialize, Validate)]
struct Ping {
    #[validate(range(min = 1, max = 3, message = "n must be 1 to 3"))]
    n: u8,
}

async fn ping(ValidatedJson(p): ValidatedJson<Ping>) -> String {
    format!("n={}", p.n)
}

async fn call(body: &str, content_type: Option<&str>) -> (StatusCode, String) {
    let mut b = Request::builder().method("POST").uri("/ping");
    if let Some(ct) = content_type {
        b = b.header(header::CONTENT_TYPE, ct);
    }
    let req = b.body(Body::from(body.to_string())).unwrap();
    let res = Router::new()
        .route("/ping", post(ping))
        .oneshot(req)
        .await
        .unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn a_valid_body_reaches_the_handler() {
    let (status, body) = call(r#"{"n":2}"#, Some("application/json")).await;
    assert_eq!((status, body.as_str()), (StatusCode::OK, "n=2"));
}

#[tokio::test]
async fn a_rule_failure_is_the_422_errors_object() {
    let (status, body) = call(r#"{"n":9}"#, Some("application/json")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, r#"{"errors":{"n":["n must be 1 to 3"]}}"#);
}

#[tokio::test]
async fn json_rejections_pass_through() {
    let (s, _) = call("{", Some("application/json")).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = call(r#"{"n":2}"#, None).await;
    assert_eq!(s, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
