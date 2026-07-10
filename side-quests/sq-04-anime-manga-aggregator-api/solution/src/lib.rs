//! A small anime/manga aggregator API that combines the three big Phase 4
//! ideas in one lower-stakes project: a cache-aside layer in front of an
//! "expensive" lookup, a token-bucket rate limiter, and `tracing`-based
//! structured logging. See `../README.md` and `SOLUTION.md` for the tour.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Whether a title is an anime or a manga — shown alongside the aggregated
/// info.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TitleKind {
    Anime,
    Manga,
}

/// Aggregated info about one title. In a production version of this
/// service, this data (and the lookup that produces it — see
/// [`fetch_title_from_source`]) would come from an upstream aggregator like
/// the [Jikan API](https://jikan.moe) (an unofficial MyAnimeList API) via
/// `reqwest`, not a hardcoded map — see `README.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TitleInfo {
    pub id: u64,
    pub title: String,
    pub kind: TitleKind,
    pub synopsis: String,
    /// Episode count for anime, chapter count for manga.
    pub unit_count: u32,
}

/// The seed dataset standing in for a real upstream source. Given, fully
/// implemented.
pub fn seed_titles() -> HashMap<u64, TitleInfo> {
    let raw = [
        (
            1,
            "Frieren: Beyond Journey's End",
            TitleKind::Anime,
            "An elf mage confronts what outliving her adventuring party actually means.",
            28,
        ),
        (
            2,
            "Solo Leveling",
            TitleKind::Manga,
            "The weakest hunter alive gets a second, much stranger, chance at power.",
            179,
        ),
        (
            3,
            "Attack on Titan",
            TitleKind::Anime,
            "Humanity's last cities live behind walls built to keep the Titans out.",
            87,
        ),
        (
            4,
            "Vinland Saga",
            TitleKind::Manga,
            "A Viking raider's path from vengeance toward something else entirely.",
            216,
        ),
        (
            5,
            "Chainsaw Man",
            TitleKind::Anime,
            "A devil hunter with a chainsaw for a heart takes on Tokyo's worst devils.",
            12,
        ),
    ];

    raw.into_iter()
        .map(|(id, title, kind, synopsis, unit_count)| {
            (
                id,
                TitleInfo {
                    id,
                    title: title.to_string(),
                    kind,
                    synopsis: synopsis.to_string(),
                    unit_count,
                },
            )
        })
        .collect()
}

/// Stands in for the "expensive" real lookup this API fronts — a network
/// call to an upstream aggregation API, a slow join across a few tables,
/// whatever. Sleeps for `latency` to simulate that cost, then returns the
/// seeded info for `id` (or `None` if no such title exists). Every
/// cache-miss request pays this cost; that's the entire reason
/// [`get_or_fetch`] exists.
pub async fn fetch_title_from_source(
    titles: &HashMap<u64, TitleInfo>,
    id: u64,
    latency: Duration,
) -> Option<TitleInfo> {
    tokio::time::sleep(latency).await;
    titles.get(&id).cloned()
}

/// A source of "now," injectable so cache-TTL tests can control time
/// without ever calling `sleep` — same idea as the cache-aside lesson's
/// `Clock` trait
/// (`phase4-backend-advanced/01-caching-with-redis/01-cache-aside-ttl-invalidation`).
/// Object-safe (no `async fn`, no generic methods) so it can live behind
/// `Arc<dyn Clock>` and keep [`TtlCache`] a plain, non-generic type — handy
/// because a non-generic cache means a non-generic `AppState`, which means
/// a plain `Router` instead of a `Router<S>` threaded through every
/// handler signature.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// The real clock — wraps `Instant::now()`. What production code uses.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock tests can move forward on demand with `advance`, so a TTL test
/// can assert "this entry is expired" without waiting out the TTL in real
/// time.
pub struct FakeClock {
    now: Mutex<Instant>,
}

impl FakeClock {
    pub fn new() -> Self {
        Self {
            now: Mutex::new(Instant::now()),
        }
    }

    pub fn advance(&self, by: Duration) {
        let mut now = self.now.lock().unwrap();
        *now += by;
    }
}

impl Default for FakeClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap()
    }
}

/// A `HashMap`-backed, TTL-expiring cache of `TitleInfo` by id — the
/// in-memory stand-in for Redis, same cache-aside shape as
/// `InMemoryCache` from the Phase 4 caching lesson. Swapping this for a
/// real `redis`-backed implementation (see that lesson's `RedisCache`)
/// would only require implementing the same `get`/`set` shape against a
/// real connection; nothing in [`get_or_fetch`] or the axum handlers would
/// need to change.
pub struct TtlCache {
    clock: Arc<dyn Clock>,
    store: Mutex<HashMap<u64, (TitleInfo, Instant)>>,
    ttl: Duration,
}

impl TtlCache {
    pub fn new(ttl: Duration) -> Self {
        Self::with_clock(Arc::new(SystemClock), ttl)
    }

    pub fn with_clock(clock: Arc<dyn Clock>, ttl: Duration) -> Self {
        Self {
            clock,
            store: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Returns `Some(info)` on a hit — present *and* not yet expired — or
    /// `None` on a miss (including an expired entry; from the caller's
    /// point of view, expired and absent look identical).
    pub fn get(&self, id: u64) -> Option<TitleInfo> {
        let store = self.store.lock().unwrap();
        match store.get(&id) {
            Some((info, expires_at)) if self.clock.now() < *expires_at => Some(info.clone()),
            _ => None,
        }
    }

    /// Stores `info` under `id`, to be treated as a miss after `self.ttl`
    /// elapses from now.
    pub fn set(&self, id: u64, info: TitleInfo) {
        let mut store = self.store.lock().unwrap();
        store.insert(id, (info, self.clock.now() + self.ttl));
    }
}

/// The cache-aside read path: check the cache; on a hit, return
/// immediately (`compute` never runs — no simulated latency paid at all).
/// On a miss, run `compute`, cache whatever it returns (if anything — a
/// title that genuinely doesn't exist has nothing worth caching), and
/// return it. Returns `(result, was_hit)` so callers can log whether this
/// particular request was a cache hit or miss. `compute` is generic over
/// `FnOnce() -> Fut` rather than an already-evaluated value, for the same
/// reason as the caching lesson's `get_or_set`: laziness is what lets a hit
/// skip the expensive work entirely instead of paying for it and throwing
/// the result away.
pub async fn get_or_fetch<F, Fut>(
    cache: &TtlCache,
    id: u64,
    compute: F,
) -> (Option<TitleInfo>, bool)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<TitleInfo>>,
{
    if let Some(info) = cache.get(id) {
        return (Some(info), true);
    }

    match compute().await {
        Some(info) => {
            cache.set(id, info.clone());
            (Some(info), false)
        }
        None => (None, false),
    }
}

/// A token bucket: `capacity` tokens max, refilling at `refill_rate` tokens
/// per second, starting full. Reused/adapted from
/// `phase4-backend-advanced/02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit`
/// — same algorithm, same reasoning (`tokens` is `f64` so a slow refill
/// rate still accumulates fractional progress instead of losing it to
/// integer truncation; `now: Instant` is an explicit parameter so tests
/// control elapsed time exactly, no real `sleep` required).
pub struct TokenBucket {
    capacity: f64,
    refill_rate: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64, now: Instant) -> Self {
        Self {
            capacity: capacity as f64,
            refill_rate,
            tokens: capacity as f64,
            last_refill: now,
        }
    }

    /// Advances `tokens` by however many seconds have elapsed since
    /// `last_refill`, times `refill_rate`, capped at `capacity`. Updates
    /// `last_refill` to `now` unconditionally, so the next call measures
    /// elapsed time from here, not from some earlier call.
    fn refill(&mut self, now: Instant) {
        let elapsed = now
            .saturating_duration_since(self.last_refill)
            .as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    /// Refills based on elapsed time, then attempts to take one token. On
    /// success, `tokens` decreases by 1 and this returns `true`. On failure
    /// (bucket empty), `tokens` is left unchanged and this returns `false`.
    pub fn try_acquire(&mut self, now: Instant) -> bool {
        self.refill(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Shared state for the axum app.
#[derive(Clone)]
pub struct AppState {
    pub titles: Arc<HashMap<u64, TitleInfo>>,
    pub cache: Arc<TtlCache>,
    pub limiter: Arc<Mutex<TokenBucket>>,
    /// How long [`fetch_title_from_source`] pretends the "expensive"
    /// lookup takes. Kept on `AppState` (rather than hardcoded) so tests
    /// can use a near-zero latency and stay fast.
    pub latency: Duration,
}

/// `GET /titles` — list every known title, no caching involved (the seed
/// dataset is already sitting in memory, so there's no "expensive" work to
/// cache in front of here).
pub async fn list_titles(State(state): State<AppState>) -> Json<Vec<TitleInfo>> {
    let mut items: Vec<TitleInfo> = state.titles.values().cloned().collect();
    items.sort_by_key(|t| t.id);
    Json(items)
}

/// The response body for `GET /titles/{id}` — the title info plus whether
/// this particular request was served from cache.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TitleResponse {
    #[serde(flatten)]
    pub info: TitleInfo,
    pub cache_hit: bool,
}

/// `GET /titles/{id}` — the cache-aside-fronted lookup. Given, fully
/// implemented: it wires [`get_or_fetch`] up to [`fetch_title_from_source`],
/// times the whole thing, and logs a structured `tracing` event with the
/// method, path, cache hit/miss, and latency for every request — the
/// "observability" leg of this project. Everything *inside* `get_or_fetch`
/// and `TtlCache` is the exercise; this handler is just wiring around it.
pub async fn get_title(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<TitleResponse>, StatusCode> {
    let start = Instant::now();
    let titles = state.titles.clone();
    let latency = state.latency;

    let (result, cache_hit) = get_or_fetch(&state.cache, id, || async move {
        fetch_title_from_source(&titles, id, latency).await
    })
    .await;

    let elapsed = start.elapsed();
    tracing::info!(
        method = "GET",
        path = %format!("/titles/{id}"),
        cache_hit,
        latency_ms = elapsed.as_millis(),
        "handled request"
    );

    match result {
        Some(info) => Ok(Json(TitleResponse { info, cache_hit })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Global rate-limiting middleware: every request must acquire a token
/// from the shared bucket, or it's rejected with `429 Too Many Requests`
/// before it ever reaches a handler. Given, fully implemented — reuses the
/// exact `TokenBucket` algorithm above (the part you implement is its
/// `refill` math, not this wiring). In a real service you'd usually prefer
/// `tower::limit::RateLimitLayer` wired in as a `Router::layer(...)` (see
/// the token-bucket lesson's README for why); this hand-rolled version
/// exists so you can see exactly what that layer does for you under the
/// hood, and so this side-quest can log a request-specific `429` message
/// instead of `tower`'s generic one.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let allowed = {
        let mut bucket = state.limiter.lock().unwrap();
        bucket.try_acquire(Instant::now())
    };

    if !allowed {
        tracing::warn!(path = %request.uri().path(), "rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate limit exceeded, try again shortly" })),
        )
            .into_response();
    }

    next.run(request).await
}

/// `/titles` lists everything; `/titles/{id}` is the cache-aside-fronted
/// single lookup. `TraceLayer` (from `tower-http`, already a workspace
/// dependency used for exactly this in real lessons) gives every request a
/// generic method/path/status/latency log line "for free"; the rate
/// limiter sits inside it so even a rejected (`429`) request still gets
/// traced. Given, fully implemented.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/titles", get(list_titles))
        .route("/titles/{id}", get(get_title))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}
