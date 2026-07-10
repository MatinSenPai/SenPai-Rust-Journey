//! Drives the whole stack through `tower::ServiceExt::oneshot`, same
//! pattern as every other axum lesson in this repo. Shares
//! `p5_07_01_urls` with `store_test.rs`, so every test here carries the
//! same `#[serial(p5_07_01_url_shortener_db)]` tag — see that file's doc
//! comment for why the tag must match across files, not just within one.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

use p5_07_01_design_a_url_shortener::{app, UrlShortenerStore};

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

    sqlx::query("CREATE SEQUENCE IF NOT EXISTS p5_07_01_urls_id_seq")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p5_07_01_urls (\
            id BIGINT PRIMARY KEY, \
            short_code TEXT UNIQUE NOT NULL, \
            original_url TEXT NOT NULL, \
            click_count BIGINT NOT NULL DEFAULT 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM p5_07_01_urls")
        .execute(&pool)
        .await
        .unwrap();

    app(Arc::new(UrlShortenerStore::new(pool)))
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn shorten_then_redirect() {
    let router = fresh_app().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"url": "https://example.com/some/page"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    let code = created["short_code"].as_str().unwrap().to_string();

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/{code}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(location, "https://example.com/some/page");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn shortening_an_invalid_url_returns_400() {
    let router = fresh_app().await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("content-type", "application/json")
                .body(Body::from(json!({"url": "ftp not a url"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn redirecting_a_missing_code_returns_404() {
    let router = fresh_app().await;

    let response = router
        .oneshot(
            Request::builder()
                .uri("/doesnotexist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn stats_reflect_redirect_visits() {
    let router = fresh_app().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"url": "https://example.com/stats-me"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = json_body(response).await;
    let code = created["short_code"].as_str().unwrap().to_string();

    router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{code}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/{code}/stats"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let stats = json_body(response).await;
    assert_eq!(stats["click_count"], 1);
}
