//! Tests for the "Build" rung: `PUT /anime/{id}` as a full replacement.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_02_03_anime_catalog_crud_in_memory_solution::{app, AnimeStore};

async fn send(
    store: &Arc<AnimeStore>,
    method: &str,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app(store.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn put_replaces_every_field_and_keeps_the_id() {
    let store = Arc::new(AnimeStore::default());
    let frieren = json!({"title": "Frieren", "status": "watching", "rating": 9});
    send(&store, "POST", "/anime", frieren).await;

    let replacement = json!({"title": "Frieren: Beyond Journey's End", "status": "completed"});
    let (status, body) = send(&store, "PUT", "/anime/1", replacement).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"id": 1, "title": "Frieren: Beyond Journey's End", "status": "completed", "rating": null})
    );
}

#[tokio::test]
async fn the_same_put_twice_gives_the_same_state_and_the_same_answer() {
    let store = Arc::new(AnimeStore::default());
    let frieren = json!({"title": "Frieren", "status": "watching", "rating": 9});
    send(&store, "POST", "/anime", frieren).await;
    let replacement = json!({"title": "Frieren", "status": "completed", "rating": 10});

    let first = send(&store, "PUT", "/anime/1", replacement.clone()).await;
    let second = send(&store, "PUT", "/anime/1", replacement).await;

    assert_eq!(first, second);
    assert_eq!(first.0, StatusCode::OK);
}

#[tokio::test]
async fn put_on_a_missing_id_is_404_not_a_create() {
    let store = Arc::new(AnimeStore::default());
    let body = json!({"title": "Frieren", "status": "watching"});
    let (status, _) = send(&store, "PUT", "/anime/5", body).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(store.list(), vec![]);
}

#[tokio::test]
async fn put_with_a_missing_field_is_rejected_by_the_json_extractor() {
    let store = Arc::new(AnimeStore::default());
    let frieren = json!({"title": "Frieren", "status": "watching", "rating": 9});
    send(&store, "POST", "/anime", frieren).await;
    let (status, _) = send(&store, "PUT", "/anime/1", json!({"title": "only a title"})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
