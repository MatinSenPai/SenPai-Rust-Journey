//! The Build rung: a duplicate title is a 409 with the code `conflict`.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use p3_08_01_consistent_error_envelopes_solution::{app, ApiError, ShowStore};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn post(store: &Arc<ShowStore>, raw: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/shows")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(raw.to_string()))
        .unwrap();
    let response = app(store.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn conflict_is_409_with_its_own_code_and_text() {
    let error = ApiError::Conflict("taken".into());
    assert_eq!(error.status(), StatusCode::CONFLICT);
    assert_eq!(error.code(), "conflict");
    assert_eq!(error.message(), "taken");
}

#[tokio::test]
async fn the_same_post_twice_is_201_then_409() {
    let store = Arc::new(ShowStore::default());
    let raw = r#"{"title":"Frieren","episodes":28}"#;
    let (first, _) = post(&store, raw).await;
    assert_eq!(first, StatusCode::CREATED);
    let (second, body) = post(&store, raw).await;
    assert_eq!(second, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({"error": {
            "code": "conflict",
            "message": "a show titled \"Frieren\" already exists"
        }})
    );
}

#[tokio::test]
async fn a_conflict_does_not_use_up_an_id() {
    let store = Arc::new(ShowStore::default());
    post(&store, r#"{"title":"A","episodes":1}"#).await;
    post(&store, r#"{"title":"A","episodes":1}"#).await;
    let (_, body) = post(&store, r#"{"title":"B","episodes":1}"#).await;
    assert_eq!(body["id"], 2);
}
