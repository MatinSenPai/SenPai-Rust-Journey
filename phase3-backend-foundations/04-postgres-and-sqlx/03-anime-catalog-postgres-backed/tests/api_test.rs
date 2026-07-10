//! Drives the whole stack — router, extractors, handlers, store, real
//! Postgres — through `tower::ServiceExt::oneshot`, the same pattern as
//! module 2's in-memory `api_test.rs`.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

use p3_04_03_anime_catalog_postgres_backed::{app, AnimeStore};
use serial_test::serial;

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn fresh_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run this ignored integration test");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p3_04_03_anime (\
            id BIGSERIAL PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL, rating SMALLINT)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM p3_04_03_anime")
        .execute(&pool)
        .await
        .unwrap();

    app(Arc::new(AnimeStore::new(pool)))
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn full_crud_lifecycle() {
    let router = fresh_app().await;

    let response = router
        .clone()
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
    let id = created["id"].as_i64().unwrap();

    let response = router
        .clone()
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
    assert_eq!(updated["title"], "Frieren");

    let response = router
        .clone()
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

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/anime/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn create_with_invalid_rating_returns_400() {
    let router = fresh_app().await;

    let response = router
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
}
