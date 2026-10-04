//! Drives the whole stack (router, extractors, handlers, store) through
//! `tower::ServiceExt::oneshot`: real requests, no socket.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, HeaderMap, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_02_03_anime_catalog_crud_in_memory_solution::{app, AnimeStore};

/// What came back: status, headers, and the raw body bytes.
struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
}

/// Sends one request to a fresh router over `store`. A `Some` body goes out
/// as JSON with the matching `Content-Type`.
async fn send(store: &Arc<AnimeStore>, method: &str, uri: &str, body: Option<Value>) -> Reply {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    };
    let response = app(store.clone()).oneshot(request.unwrap()).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        headers,
        body: body.to_vec(),
    }
}

fn frieren() -> Value {
    json!({"title": "Frieren", "status": "watching", "rating": 9})
}

#[tokio::test]
async fn full_crud_lifecycle() {
    let store = Arc::new(AnimeStore::default());

    let created = send(&store, "POST", "/anime", Some(frieren())).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(created.headers[header::LOCATION], "/anime/1");
    assert_eq!(
        created.json(),
        json!({"id": 1, "title": "Frieren", "status": "watching", "rating": 9})
    );

    let listed = send(&store, "GET", "/anime", None).await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.json(), json!([created.json()]));

    let fetched = send(&store, "GET", "/anime/1", None).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.json(), created.json());

    let patched = send(
        &store,
        "PATCH",
        "/anime/1",
        Some(json!({"status": "completed"})),
    )
    .await;
    assert_eq!(patched.status, StatusCode::OK);
    assert_eq!(patched.json()["status"], "completed");
    assert_eq!(patched.json()["title"], "Frieren"); // untouched by the partial update

    let deleted = send(&store, "DELETE", "/anime/1", None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert!(deleted.body.is_empty());

    let gone = send(&store, "GET", "/anime/1", None).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_eq!(gone.json(), json!({"error": "anime not found"}));
}

#[tokio::test]
async fn create_with_invalid_rating_returns_422_with_an_error_body() {
    let store = Arc::new(AnimeStore::default());
    let body = json!({"title": "Frieren", "status": "watching", "rating": 15});

    let reply = send(&store, "POST", "/anime", Some(body)).await;

    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        reply.json(),
        json!({"error": "rating must be between 1 and 10, got 15"})
    );
    assert_eq!(send(&store, "GET", "/anime", None).await.json(), json!([]));
}

#[tokio::test]
async fn get_missing_anime_returns_404_with_an_error_body() {
    let store = Arc::new(AnimeStore::default());
    let reply = send(&store, "GET", "/anime/9999", None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(reply.json(), json!({"error": "anime not found"}));
}

#[tokio::test]
async fn patch_missing_anime_returns_404() {
    let store = Arc::new(AnimeStore::default());
    let reply = send(&store, "PATCH", "/anime/7", Some(json!({"rating": 8}))).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_with_invalid_rating_returns_422() {
    let store = Arc::new(AnimeStore::default());
    send(&store, "POST", "/anime", Some(frieren())).await;
    let reply = send(&store, "PATCH", "/anime/1", Some(json!({"rating": 0}))).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        reply.json(),
        json!({"error": "rating must be between 1 and 10, got 0"})
    );
}

#[tokio::test]
async fn deleting_twice_answers_differently_but_leaves_the_same_state() {
    let store = Arc::new(AnimeStore::default());
    send(&store, "POST", "/anime", Some(frieren())).await;

    let first = send(&store, "DELETE", "/anime/1", None).await;
    let state_after_first = send(&store, "GET", "/anime", None).await.json();
    let second = send(&store, "DELETE", "/anime/1", None).await;
    let state_after_second = send(&store, "GET", "/anime", None).await.json();

    assert_eq!(first.status, StatusCode::NO_CONTENT);
    assert_eq!(second.status, StatusCode::NOT_FOUND);
    assert_eq!(state_after_first, json!([]));
    assert_eq!(state_after_second, state_after_first);
}

#[tokio::test]
async fn list_starts_empty() {
    let store = Arc::new(AnimeStore::default());
    let reply = send(&store, "GET", "/anime", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json(), json!([]));
}
