//! Exercises `UrlShortenerStore` directly against a live Postgres. All
//! tests share the `p5_07_01_urls` table (and its backing sequence) in the
//! `taskforge` database, so every test carries
//! `#[serial(p5_07_01_url_shortener_db)]` — see
//! `phase3-backend-foundations/04-postgres-and-sqlx/02-migrations/README.md`'s
//! "Why every test carries #[serial(...)]" section for the full reasoning
//! (a real, reproducible flakiness bug this repo's own test suite hit
//! without it).

use p5_07_01_design_a_url_shortener_solution::{UrlShortenerError, UrlShortenerStore};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn fresh_store() -> UrlShortenerStore {
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

    UrlShortenerStore::new(pool)
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn shortening_a_valid_url_returns_a_short_code() {
    let store = fresh_store().await;
    let short = store
        .shorten("https://example.com/a/long/path")
        .await
        .unwrap();
    assert!(!short.short_code.is_empty());
    assert_eq!(short.original_url, "https://example.com/a/long/path");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn shortening_an_invalid_url_is_rejected() {
    let store = fresh_store().await;
    let result = store.shorten("not-a-url").await;
    assert!(matches!(result, Err(UrlShortenerError::InvalidUrl(_))));
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn resolving_a_shortened_url_returns_the_original() {
    let store = fresh_store().await;
    let short = store
        .shorten("https://example.com/resolve-me")
        .await
        .unwrap();

    let resolved = store.resolve(&short.short_code).await.unwrap();

    assert_eq!(resolved, Some("https://example.com/resolve-me".to_string()));
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn resolving_a_missing_code_returns_none() {
    let store = fresh_store().await;
    assert_eq!(store.resolve("doesnotexist").await.unwrap(), None);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn each_resolve_increments_the_click_count() {
    let store = fresh_store().await;
    let short = store.shorten("https://example.com/counted").await.unwrap();

    store.resolve(&short.short_code).await.unwrap();
    store.resolve(&short.short_code).await.unwrap();
    store.resolve(&short.short_code).await.unwrap();

    let stats = store.stats(&short.short_code).await.unwrap().unwrap();
    assert_eq!(stats.click_count, 3);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn stats_for_a_missing_code_returns_none() {
    let store = fresh_store().await;
    assert!(store.stats("doesnotexist").await.unwrap().is_none());
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_01_url_shortener_db)]
async fn shortening_the_same_url_twice_produces_two_distinct_codes() {
    let store = fresh_store().await;
    let first = store.shorten("https://example.com/dup").await.unwrap();
    let second = store.shorten("https://example.com/dup").await.unwrap();

    assert_ne!(first.short_code, second.short_code);
}
