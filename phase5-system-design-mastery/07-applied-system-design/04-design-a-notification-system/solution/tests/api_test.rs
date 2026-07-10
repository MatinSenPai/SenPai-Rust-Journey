//! Drives the REST endpoints (create/list/mark-read) through
//! `tower::ServiceExt::oneshot`. The `/stream` SSE route is exercised at
//! the `NotificationStore` level instead (see `store_test.rs`'s
//! `a_subscriber_receives_a_notification_created_after_it_subscribed`) —
//! `oneshot` sends one request and waits for one complete response, which
//! doesn't fit an endpoint whose whole point is staying open and streaming
//! indefinitely.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

use p5_07_04_design_a_notification_system_solution::{app, NotificationStore};

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
        "CREATE TABLE IF NOT EXISTS p5_07_04_notifications (\
            id BIGSERIAL PRIMARY KEY, \
            user_id TEXT NOT NULL, \
            message TEXT NOT NULL, \
            read BOOLEAN NOT NULL DEFAULT false)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM p5_07_04_notifications")
        .execute(&pool)
        .await
        .unwrap();

    app(Arc::new(NotificationStore::new(pool)))
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn create_then_list_then_mark_read() {
    let router = fresh_app().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notifications")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"user_id": "alice", "message": "hi"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    let id = created["id"].as_i64().unwrap();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/notifications/alice")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let unread = json_body(response).await;
    assert_eq!(unread.as_array().unwrap().len(), 1);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/notifications/{id}/read"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/notifications/alice")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let unread_after = json_body(response).await;
    assert_eq!(unread_after.as_array().unwrap().len(), 0);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn marking_a_missing_notification_read_returns_404() {
    let router = fresh_app().await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notifications/999999/read")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
