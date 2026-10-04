//! Tests for the "Build" rung: `GET /search?q=...&limit=...`.
//! They fail until you add the route; the spec is in the README.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use p3_02_01_routing_handlers_extractors_solution::{app, AppState};

async fn get(uri: &str) -> (StatusCode, String) {
    let response = app(AppState::default())
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn search_echoes_q_and_limit() {
    let (status, body) = get("/search?q=naruto&limit=3").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"q":"naruto","limit":3}"#);
}

#[tokio::test]
async fn search_limit_defaults_to_10() {
    let (status, body) = get("/search?q=naruto").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"q":"naruto","limit":10}"#);
}

#[tokio::test]
async fn search_decodes_percent_encoding() {
    let (_, body) = get("/search?q=one%20piece").await;
    assert_eq!(body, r#"{"q":"one piece","limit":10}"#);
}

#[tokio::test]
async fn search_without_q_is_rejected_with_400() {
    let (status, _) = get("/search?limit=3").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_with_a_non_numeric_limit_is_rejected_with_400() {
    let (status, _) = get("/search?q=a&limit=many").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
