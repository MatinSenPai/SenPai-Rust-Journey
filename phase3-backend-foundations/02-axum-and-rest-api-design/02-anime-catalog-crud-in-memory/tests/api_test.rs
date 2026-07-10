//! Drives the whole stack — router, extractors, handlers, store — through
//! `tower::ServiceExt::oneshot`, exactly like `02-01`'s `routes_test.rs`.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_02_02_anime_catalog_crud_in_memory::{app, AnimeStore};

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn fresh_app() -> axum::Router {
    app(Arc::new(AnimeStore::default()))
}

#[tokio::test]
async fn full_crud_lifecycle() {
    let store = Arc::new(AnimeStore::default());

    // CREATE
    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/anime")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Frieren", "status": "watching", "rating": 9}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    assert_eq!(created["title"], "Frieren");
    let id = created["id"].as_u64().unwrap();

    // LIST
    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .uri("/anime")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = json_body(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // GET one
    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/anime/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["status"], "watching");

    // PATCH
    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/anime/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"status": "completed"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["status"], "completed");
    assert_eq!(updated["title"], "Frieren"); // untouched by the partial update

    // DELETE
    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/anime/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // GET after delete -> 404
    let response = app(store)
        .oneshot(
            Request::builder()
                .uri(format!("/anime/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let error = json_body(response).await;
    assert_eq!(error["error"], "anime not found");
}

#[tokio::test]
async fn create_with_invalid_rating_returns_400_with_an_error_body() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/anime")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Frieren", "status": "watching", "rating": 15}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = json_body(response).await;
    assert!(error["error"]
        .as_str()
        .unwrap()
        .contains("between 1 and 10"));
}

#[tokio::test]
async fn get_missing_anime_returns_404() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .uri("/anime/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_starts_empty() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .uri("/anime")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let list = json_body(response).await;
    assert_eq!(list, json!([]));
}
