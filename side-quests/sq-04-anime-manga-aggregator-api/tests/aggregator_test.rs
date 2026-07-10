//! Exercises the cache, the cache-aside orchestration, and the rate
//! limiter — the three pieces this side-quest's `todo!()`s live in — plus
//! the axum app wiring them together.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sq_04_anime_manga_aggregator_api::{
    app, get_or_fetch, seed_titles, AppState, FakeClock, TitleInfo, TitleKind, TokenBucket,
    TtlCache,
};
use tower::ServiceExt;

fn sample_title() -> TitleInfo {
    TitleInfo {
        id: 1,
        title: "Frieren: Beyond Journey's End".to_string(),
        kind: TitleKind::Anime,
        synopsis: "An elf mage confronts what outliving her party means.".to_string(),
        unit_count: 28,
    }
}

#[test]
fn cache_miss_on_a_fresh_cache() {
    let cache = TtlCache::new(Duration::from_secs(60));
    assert_eq!(cache.get(1), None);
}

#[test]
fn cache_hit_after_set() {
    let cache = TtlCache::new(Duration::from_secs(60));
    cache.set(1, sample_title());
    assert_eq!(cache.get(1), Some(sample_title()));
}

#[test]
fn cache_entry_expires_after_its_ttl() {
    let clock = Arc::new(FakeClock::new());
    let cache = TtlCache::with_clock(clock.clone(), Duration::from_secs(60));
    cache.set(1, sample_title());

    clock.advance(Duration::from_secs(61));

    assert_eq!(cache.get(1), None);
}

#[test]
fn cache_entry_is_still_a_hit_just_before_its_ttl() {
    let clock = Arc::new(FakeClock::new());
    let cache = TtlCache::with_clock(clock.clone(), Duration::from_secs(60));
    cache.set(1, sample_title());

    clock.advance(Duration::from_secs(59));

    assert_eq!(cache.get(1), Some(sample_title()));
}

#[tokio::test]
async fn get_or_fetch_computes_and_caches_on_a_miss() {
    let cache = TtlCache::new(Duration::from_secs(60));
    let calls = AtomicUsize::new(0);

    let (result, cache_hit) = get_or_fetch(&cache, 1, || async {
        calls.fetch_add(1, Ordering::SeqCst);
        Some(sample_title())
    })
    .await;

    assert_eq!(result, Some(sample_title()));
    assert!(!cache_hit);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(cache.get(1), Some(sample_title()));
}

#[tokio::test]
async fn get_or_fetch_never_calls_compute_on_a_hit() {
    let cache = TtlCache::new(Duration::from_secs(60));
    cache.set(1, sample_title());
    let calls = AtomicUsize::new(0);

    let (result, cache_hit) = get_or_fetch(&cache, 1, || async {
        calls.fetch_add(1, Ordering::SeqCst);
        Some(sample_title())
    })
    .await;

    assert_eq!(result, Some(sample_title()));
    assert!(cache_hit);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "a cache hit must never run the expensive compute closure"
    );
}

#[tokio::test]
async fn get_or_fetch_caches_nothing_for_a_title_that_does_not_exist() {
    let cache = TtlCache::new(Duration::from_secs(60));

    let (result, cache_hit) = get_or_fetch(&cache, 999, || async { None }).await;

    assert_eq!(result, None);
    assert!(!cache_hit);
    assert_eq!(cache.get(999), None);
}

#[test]
fn token_bucket_starts_full_and_drains_one_token_per_acquire() {
    let now = Instant::now();
    let mut bucket = TokenBucket::new(3, 1.0, now);

    assert!(bucket.try_acquire(now));
    assert!(bucket.try_acquire(now));
    assert!(bucket.try_acquire(now));
    assert!(
        !bucket.try_acquire(now),
        "a 4th immediate acquire against a 3-token bucket must fail"
    );
}

#[test]
fn token_bucket_refills_over_time_up_to_capacity() {
    let now = Instant::now();
    let mut bucket = TokenBucket::new(2, 1.0, now);

    assert!(bucket.try_acquire(now));
    assert!(bucket.try_acquire(now));
    assert!(!bucket.try_acquire(now));

    let later = now + Duration::from_secs(1);
    assert!(
        bucket.try_acquire(later),
        "1 second at a 1 token/sec refill rate should have replenished exactly 1 token"
    );
    assert!(!bucket.try_acquire(later));
}

fn test_state() -> AppState {
    AppState {
        titles: Arc::new(seed_titles()),
        cache: Arc::new(TtlCache::new(Duration::from_secs(60))),
        limiter: Arc::new(Mutex::new(TokenBucket::new(1000, 1000.0, Instant::now()))),
        latency: Duration::from_millis(1),
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn listing_titles_returns_every_seeded_title() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/titles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body.as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn getting_a_title_twice_is_a_miss_then_a_hit() {
    let router = app(test_state());

    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/titles/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = body_json(first).await;
    assert_eq!(first_body["cache_hit"], false);

    let second = router
        .oneshot(
            Request::builder()
                .uri("/titles/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = body_json(second).await;
    assert_eq!(second_body["cache_hit"], true);
    assert_eq!(second_body["title"], first_body["title"]);
}

#[tokio::test]
async fn getting_a_missing_title_returns_404() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/titles/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn exhausting_the_rate_limiter_returns_429() {
    let mut state = test_state();
    state.limiter = Arc::new(Mutex::new(TokenBucket::new(1, 0.0, Instant::now())));
    let router = app(state);

    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/titles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = router
        .oneshot(
            Request::builder()
                .uri("/titles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
}
