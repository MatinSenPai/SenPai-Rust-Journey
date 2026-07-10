//! Cache-aside, TTL, and invalidation over a small `Cache` trait — see
//! `README.md` for the theory. Two implementations of `Cache` live here:
//! `InMemoryCache` (what you implement, backs every non-`#[ignore]`d test)
//! and `RedisCache` (given, the real thing).

use std::collections::HashMap;
use std::future::Future;
use std::time::{Duration, Instant};

/// Everything that can go wrong talking to a cache backend. `InMemoryCache`
/// never actually fails (it's just a `HashMap`), but the trait still
/// returns `Result` because a *real* backend (network hop to Redis) can —
/// and code written against the trait should handle that from day one
/// rather than bolting error handling on later.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache backend error: {0}")]
    Backend(String),
}

/// A key-value cache with expiry. Deliberately small: just enough to
/// implement cache-aside on top of. Note these are `async fn`s directly in
/// the trait (stable since Rust 1.75) rather than a `#[async_trait]` macro
/// — that's only needed when you must call the trait through `dyn Cache`
/// (trait objects), which this lesson never does; every function here is
/// generic over `C: Cache` instead, so the plain syntax works.
///
/// Note these return `impl Future<...>` without a `+ Send` bound — this
/// trait is written for the "await it directly, generic caller" pattern
/// used throughout this lesson (`get_or_set`), not for spawning cache
/// calls onto other tokio tasks. Requiring `+ Send` would force every
/// implementor's fields to be `Sync` too (a future that captures `&self`
/// is only `Send` if `Self: Sync`), which `FakeClock`'s interior-mutable
/// `Cell` deliberately is not — `Cell` is single-threaded-only by design.
pub trait Cache {
    /// Returns `Ok(Some(value))` on a hit, `Ok(None)` on a miss (including
    /// an expired entry — from the caller's point of view, expired and
    /// absent look identical).
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<String>, CacheError>>;

    /// Stores `value` under `key`, to be treated as a miss after `ttl`
    /// elapses.
    fn set(
        &mut self,
        key: &str,
        value: String,
        ttl: Duration,
    ) -> impl Future<Output = Result<(), CacheError>>;

    /// Removes `key` immediately, regardless of its TTL. This is the
    /// "write path" half of cache-aside: call this the moment the
    /// underlying data changes.
    fn invalidate(&mut self, key: &str) -> impl Future<Output = Result<(), CacheError>>;
}

/// Builds a cache key as `"{namespace}:{id}"`, e.g. `cache_key("user", "42")
/// == "user:42"`. Grouping related entries under a shared namespace prefix
/// is what makes "invalidate everything about user 42" or "these are all
/// homepage-feed entries" a tractable question later, and prevents two
/// unrelated features from accidentally colliding on the same bare key.
pub fn cache_key(namespace: &str, id: &str) -> String {
    todo!("format!(\"{{namespace}}:{{id}}\")")
}

/// A source of "now," injectable so tests can control time without ever
/// calling `sleep`. `SystemClock` is what production code uses;
/// `FakeClock` is what tests use.
pub trait Clock {
    fn now(&self) -> Instant;
}

/// The real clock — wraps `Instant::now()`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock tests can move forward on demand with `advance`, so a TTL test
/// can assert "this entry is expired" without actually waiting out the
/// TTL in real time (which would make the test suite slow and, at short
/// enough TTLs, flaky).
pub struct FakeClock {
    now: std::cell::Cell<Instant>,
}

impl FakeClock {
    pub fn new() -> Self {
        Self {
            now: std::cell::Cell::new(Instant::now()),
        }
    }

    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get() + by);
    }
}

impl Default for FakeClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.now.get()
    }
}

/// An in-memory, `HashMap`-backed `Cache`. This is what every deterministic
/// test in this lesson (and in the side-quest) runs against — no network,
/// no external process, just the cache-aside *logic* under test. Generic
/// over `CL: Clock` so tests can inject a `FakeClock`; production code
/// (were you to actually use this backend, e.g. for a single-process
/// dev-mode cache) would use the default `SystemClock`.
pub struct InMemoryCache<CL: Clock = SystemClock> {
    clock: CL,
    store: HashMap<String, (String, Instant)>,
}

impl InMemoryCache<SystemClock> {
    pub fn new() -> Self {
        Self {
            clock: SystemClock,
            store: HashMap::new(),
        }
    }
}

impl Default for InMemoryCache<SystemClock> {
    fn default() -> Self {
        Self::new()
    }
}

impl<CL: Clock> InMemoryCache<CL> {
    pub fn with_clock(clock: CL) -> Self {
        Self {
            clock,
            store: HashMap::new(),
        }
    }
}

impl<CL: Clock> Cache for InMemoryCache<CL> {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        todo!(
            "look up `key` in self.store; if present AND self.clock.now() is \
             before the stored expiry, return Ok(Some(value.clone())); \
             otherwise (absent OR expired) return Ok(None)"
        )
    }

    async fn set(&mut self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError> {
        todo!(
            "insert (value, self.clock.now() + ttl) into self.store under `key`, \
             return Ok(())"
        )
    }

    async fn invalidate(&mut self, key: &str) -> Result<(), CacheError> {
        todo!("remove `key` from self.store, return Ok(())")
    }
}

/// The cache-aside read path: try the cache first; on a miss, run `compute`
/// to get the real value, store it under `key` with `ttl`, and return it.
/// On a hit, `compute` must NOT run at all — that's the entire point of
/// caching (see the `compute_only_runs_once_on_repeated_hits` test).
pub async fn get_or_set<C, F, Fut>(
    cache: &mut C,
    key: &str,
    ttl: Duration,
    compute: F,
) -> Result<String, CacheError>
where
    C: Cache,
    F: FnOnce() -> Fut,
    Fut: Future<Output = String>,
{
    todo!(
        "if cache.get(key).await? is Some(value), return Ok(value); otherwise \
         call compute().await, cache.set(key, value.clone(), ttl).await?, then \
         return Ok(value)"
    )
}

/// A real Redis-backed `Cache`, given fully implemented so you can see what
/// production code against the `redis` crate looks like. `#[ignore]`d tests
/// below exercise this against a genuinely live Redis — see `README.md`
/// for how to run them.
pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    /// `addr` is a Redis connection URL, e.g. `"redis://127.0.0.1/"`.
    pub fn new(addr: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(addr).map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(Self { client })
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection, CacheError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))
    }
}

impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        use redis::AsyncCommands;
        let mut conn = self.connection().await?;
        conn.get(key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))
    }

    async fn set(&mut self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError> {
        use redis::AsyncCommands;
        let mut conn = self.connection().await?;
        let ttl_secs = ttl.as_secs().max(1);
        let _: () = conn
            .set_ex(key, value, ttl_secs)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(())
    }

    async fn invalidate(&mut self, key: &str) -> Result<(), CacheError> {
        use redis::AsyncCommands;
        let mut conn = self.connection().await?;
        let _: () = conn
            .del(key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn builds_namespaced_keys() {
        assert_eq!(cache_key("user", "42"), "user:42");
        assert_eq!(cache_key("homepage-feed", "page-1"), "homepage-feed:page-1");
    }

    #[tokio::test]
    async fn miss_then_hit_round_trips_through_in_memory_cache() {
        let mut cache = InMemoryCache::new();
        let key = cache_key("user", "42");

        assert_eq!(cache.get(&key).await.unwrap(), None);

        cache
            .set(&key, "matin".to_string(), Duration::from_secs(60))
            .await
            .unwrap();

        assert_eq!(cache.get(&key).await.unwrap(), Some("matin".to_string()));
    }

    #[tokio::test]
    async fn invalidate_forces_a_miss() {
        let mut cache = InMemoryCache::new();
        let key = cache_key("user", "42");
        cache
            .set(&key, "matin".to_string(), Duration::from_secs(60))
            .await
            .unwrap();

        cache.invalidate(&key).await.unwrap();

        assert_eq!(cache.get(&key).await.unwrap(), None);
    }

    #[tokio::test]
    async fn entry_expires_after_its_ttl() {
        let clock = FakeClock::new();
        let mut cache = InMemoryCache::with_clock(clock);
        let key = cache_key("session", "abc");

        cache
            .set(&key, "token".to_string(), Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(cache.get(&key).await.unwrap(), Some("token".to_string()));

        // Not expired yet at 29s.
        cache.clock.advance(Duration::from_secs(29));
        assert_eq!(cache.get(&key).await.unwrap(), Some("token".to_string()));

        // Expired by 31s.
        cache.clock.advance(Duration::from_secs(2));
        assert_eq!(cache.get(&key).await.unwrap(), None);
    }

    #[tokio::test]
    async fn get_or_set_returns_the_computed_value_on_a_miss() {
        let mut cache = InMemoryCache::new();
        let value = get_or_set(
            &mut cache,
            "expensive:1",
            Duration::from_secs(60),
            || async { "computed".to_string() },
        )
        .await
        .unwrap();

        assert_eq!(value, "computed");
    }

    #[tokio::test]
    async fn get_or_set_compute_only_runs_once_on_repeated_hits() {
        let mut cache = InMemoryCache::new();
        let calls = Rc::new(Cell::new(0));

        for _ in 0..3 {
            let calls = Rc::clone(&calls);
            let value = get_or_set(
                &mut cache,
                "expensive:1",
                Duration::from_secs(60),
                || async move {
                    calls.set(calls.get() + 1);
                    "computed".to_string()
                },
            )
            .await
            .unwrap();
            assert_eq!(value, "computed");
        }

        assert_eq!(
            calls.get(),
            1,
            "compute should only run on the first (miss) call, not on subsequent hits"
        );
    }

    #[tokio::test]
    async fn get_or_set_recomputes_after_ttl_expiry() {
        let clock = FakeClock::new();
        let mut cache = InMemoryCache::with_clock(clock);
        let calls = Rc::new(Cell::new(0));

        for _ in 0..2 {
            let calls = Rc::clone(&calls);
            get_or_set(
                &mut cache,
                "expensive:2",
                Duration::from_secs(10),
                || async move {
                    calls.set(calls.get() + 1);
                    "computed".to_string()
                },
            )
            .await
            .unwrap();
        }
        assert_eq!(calls.get(), 1, "second call within the TTL should be a hit");

        cache.clock.advance(Duration::from_secs(11));
        let calls2 = Rc::clone(&calls);
        get_or_set(
            &mut cache,
            "expensive:2",
            Duration::from_secs(10),
            || async move {
                calls2.set(calls2.get() + 1);
                "computed".to_string()
            },
        )
        .await
        .unwrap();
        assert_eq!(calls.get(), 2, "call after TTL expiry should recompute");
    }

    /// Needs a real Redis running locally. Start one first:
    ///   redis-server --daemonize yes
    /// then:
    ///   cargo test -p p4-01-01-cache-aside-ttl-invalidation -- --ignored
    #[tokio::test]
    #[ignore]
    async fn redis_cache_round_trips_against_a_live_redis() {
        let mut cache = RedisCache::new("redis://127.0.0.1/").unwrap();
        let key = cache_key("integration-test", "round-trip");

        cache.invalidate(&key).await.unwrap();
        assert_eq!(cache.get(&key).await.unwrap(), None);

        cache
            .set(&key, "hello-redis".to_string(), Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            cache.get(&key).await.unwrap(),
            Some("hello-redis".to_string())
        );

        cache.invalidate(&key).await.unwrap();
        assert_eq!(cache.get(&key).await.unwrap(), None);
    }
}
