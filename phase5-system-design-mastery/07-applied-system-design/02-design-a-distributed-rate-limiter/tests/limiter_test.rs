//! Exercises `RedisRateLimiter` against a live Redis. Unlike the
//! Postgres-backed lessons elsewhere in this repo, these tests do **not**
//! need `#[serial(...)]`: each test generates its own unique Redis key (see
//! `unique_key` below), so no two tests ever touch the same key even when
//! `cargo test` runs them concurrently — a different, often simpler,
//! alternative to serializing tests when the shared backend supports cheap
//! per-test namespacing (Redis keys are free; a Postgres table's schema
//! is not).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use p5_07_02_design_a_distributed_rate_limiter::RedisRateLimiter;

fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string())
}

fn unique_key(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("ratelimit-test:{name}:{nanos}")
}

#[tokio::test]
#[ignore = "requires a live Redis reachable at REDIS_URL (default redis://127.0.0.1/)"]
async fn allows_calls_up_to_the_limit() {
    let limiter = RedisRateLimiter::new(&redis_url(), 3, Duration::from_secs(60)).unwrap();
    let key = unique_key("allows_up_to_limit");

    assert!(limiter.check(&key).await.unwrap());
    assert!(limiter.check(&key).await.unwrap());
    assert!(limiter.check(&key).await.unwrap());
}

#[tokio::test]
#[ignore = "requires a live Redis reachable at REDIS_URL (default redis://127.0.0.1/)"]
async fn rejects_the_call_that_exceeds_the_limit() {
    let limiter = RedisRateLimiter::new(&redis_url(), 2, Duration::from_secs(60)).unwrap();
    let key = unique_key("rejects_over_limit");

    assert!(limiter.check(&key).await.unwrap());
    assert!(limiter.check(&key).await.unwrap());
    assert!(!limiter.check(&key).await.unwrap());
}

#[tokio::test]
#[ignore = "requires a live Redis reachable at REDIS_URL (default redis://127.0.0.1/)"]
async fn different_keys_have_independent_limits() {
    let limiter = RedisRateLimiter::new(&redis_url(), 1, Duration::from_secs(60)).unwrap();
    let key_a = unique_key("independent_a");
    let key_b = unique_key("independent_b");

    assert!(limiter.check(&key_a).await.unwrap());
    assert!(!limiter.check(&key_a).await.unwrap());
    // key_b has never been called — it gets its own fresh limit, untouched
    // by key_a's usage.
    assert!(limiter.check(&key_b).await.unwrap());
}

#[tokio::test]
#[ignore = "requires a live Redis reachable at REDIS_URL (default redis://127.0.0.1/)"]
async fn the_window_resets_after_it_expires() {
    let limiter = RedisRateLimiter::new(&redis_url(), 1, Duration::from_secs(1)).unwrap();
    let key = unique_key("window_resets");

    assert!(limiter.check(&key).await.unwrap());
    assert!(!limiter.check(&key).await.unwrap());

    tokio::time::sleep(Duration::from_millis(1100)).await;

    assert!(
        limiter.check(&key).await.unwrap(),
        "a new window should have started after the 1-second limit elapsed"
    );
}

#[tokio::test]
#[ignore = "requires a live Redis reachable at REDIS_URL (default redis://127.0.0.1/)"]
async fn concurrent_callers_never_exceed_the_limit() {
    // The property FOR UPDATE SKIP LOCKED proved for Postgres in the toy
    // queue lesson, proven here for Redis: many concurrent callers racing
    // against the same key never let more than `limit` calls through, even
    // though every caller is a separate connection issuing requests at
    // nearly the same instant.
    let limiter = std::sync::Arc::new(
        RedisRateLimiter::new(&redis_url(), 5, Duration::from_secs(60)).unwrap(),
    );
    let key = unique_key("concurrent_callers");

    let mut handles = Vec::new();
    for _ in 0..20 {
        let limiter = limiter.clone();
        let key = key.clone();
        handles.push(tokio::spawn(
            async move { limiter.check(&key).await.unwrap() },
        ));
    }

    let mut allowed_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            allowed_count += 1;
        }
    }

    assert_eq!(allowed_count, 5);
}
