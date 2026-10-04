//! The whole stack (router, extractor, validation, store) through
//! `tower::ServiceExt::oneshot`: real requests, no socket.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_03_02_validation_solution::{app, ReviewStore};

/// Sends one request to a fresh router over `store`; returns the status,
/// the `Content-Type` header (if any) and the raw body text.
async fn send(
    store: &Arc<ReviewStore>,
    method: &str,
    body: Option<&str>,
    content_type: Option<&str>,
) -> (StatusCode, Option<String>, String) {
    let mut builder = Request::builder().method(method).uri("/reviews");
    if let Some(ct) = content_type {
        builder = builder.header(header::CONTENT_TYPE, ct);
    }
    let request = builder
        .body(Body::from(body.unwrap_or("").to_string()))
        .unwrap();
    let response = app(store.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let ct = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string());
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, ct, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn post(store: &Arc<ReviewStore>, body: Value) -> (StatusCode, Option<String>, String) {
    send(
        store,
        "POST",
        Some(&body.to_string()),
        Some("application/json"),
    )
    .await
}

fn good() -> Value {
    json!({
        "title": "Frieren",
        "rating": 9,
        "reviewer": {"handle": "matin_01", "email": "matin@example.com"}
    })
}

#[tokio::test]
async fn a_valid_review_is_created_and_listed() {
    let store = Arc::new(ReviewStore::default());
    let (status, _, body) = post(&store, good()).await;
    assert_eq!(status, StatusCode::CREATED);
    let created: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        created,
        json!({"id": 1, "title": "Frieren", "rating": 9, "reviewer": "matin_01"})
    );

    let (status, _, body) = send(&store, "GET", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&body).unwrap(),
        json!([created])
    );
}

#[tokio::test]
async fn a_bad_rating_is_a_422_with_a_field_keyed_json_body() {
    let store = Arc::new(ReviewStore::default());
    let mut bad = good();
    bad["rating"] = json!(15);
    let (status, ct, body) = post(&store, bad).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(ct.as_deref(), Some("application/json"));
    assert_eq!(
        body,
        r#"{"errors":{"rating":["rating must be between 1 and 10"]}}"#
    );
}

#[tokio::test]
async fn nested_and_list_errors_come_back_under_their_paths() {
    let store = Arc::new(ReviewStore::default());
    let mut bad = good();
    bad["reviewer"]["email"] = json!("nope");
    bad["notes"] = json!([{"episode": 1, "text": "ok"}, {"episode": 0, "text": "x"}]);
    let (status, _, body) = post(&store, bad).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        v,
        json!({"errors": {"reviewer.email": ["email"], "notes[1].episode": ["range"]}})
    );
}

#[tokio::test]
async fn a_rejected_review_is_not_stored_and_uses_up_no_id() {
    let store = Arc::new(ReviewStore::default());
    let mut bad = good();
    bad["title"] = json!("");
    assert_eq!(post(&store, bad).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(store.list().is_empty());
    let (_, _, body) = post(&store, good()).await;
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["id"], 1);
}

#[tokio::test]
async fn broken_json_is_a_400_and_a_wrong_shape_is_axums_own_422() {
    let store = Arc::new(ReviewStore::default());
    let (status, _, _) = send(&store, "POST", Some("{"), Some("application/json")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Right JSON, wrong shape: `rating` is a string. Rejected before any
    // rule runs, with axum's plain-text body, not the `errors` object.
    let mut wrong = good();
    wrong["rating"] = json!("nine");
    let (status, ct, body) = post(&store, wrong).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(ct.unwrap().starts_with("text/plain"));
    assert!(!body.contains("\"errors\""));
}

#[tokio::test]
async fn a_missing_content_type_is_a_415() {
    let store = Arc::new(ReviewStore::default());
    let (status, _, _) = send(&store, "POST", Some(&good().to_string()), None).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
