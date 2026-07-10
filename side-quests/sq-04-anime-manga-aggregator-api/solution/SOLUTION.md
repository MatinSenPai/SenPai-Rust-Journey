# Solution

```rust
pub fn get(&self, id: u64) -> Option<TitleInfo> {
    let store = self.store.lock().unwrap();
    match store.get(&id) {
        Some((info, expires_at)) if self.clock.now() < *expires_at => Some(info.clone()),
        _ => None,
    }
}

pub fn set(&self, id: u64, info: TitleInfo) {
    let mut store = self.store.lock().unwrap();
    store.insert(id, (info, self.clock.now() + self.ttl));
}

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

fn refill(&mut self, now: Instant) {
    let elapsed = now.saturating_duration_since(self.last_refill).as_secs_f64();
    self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
    self.last_refill = now;
}
```

## Why `compute` is a closure, not a value

If `get_or_fetch` took `compute: Option<TitleInfo>` instead, the caller
would have to produce that value — meaning `await` the expensive lookup —
*before* calling `get_or_fetch` at all, on every single call, hit or miss.
The entire performance benefit of caching would evaporate: you'd still pay
`fetch_title_from_source`'s simulated latency on every request, then throw
the result away on a cache hit. Taking `compute: F where F: FnOnce() ->
Fut` instead means the expensive work is only *described*, not yet run,
when it's passed in — `get_or_fetch` only calls `compute().await` in the
branch where the cache actually missed. `get_title`'s call site makes this
concrete: `|| async move { fetch_title_from_source(...).await }` builds a
closure that, when invoked, produces a future; nothing runs until (and
unless) `get_or_fetch` decides it needs to.

## Why `Arc<dyn Clock>` instead of `Instant::now()` directly

If `TtlCache` called `Instant::now()` directly, `cache_entry_expires_after_its_ttl`
would have no way to fast-forward time — the only way to prove an entry
expires would be to actually `tokio::time::sleep` for the full TTL in the
test, which is both slow and, for anything longer than a few seconds,
impractical to test at all. `Arc<dyn Clock>` lets tests substitute
`FakeClock`, whose `now()` returns whatever the test last set via
`.advance(...)` — `cache_entry_expires_after_its_ttl` advances a `FakeClock`
by 61 seconds and immediately observes the entry as expired, with zero real
time elapsed. This is the identical pattern the Redis caching lesson's
`Clock` trait uses, for the identical reason.

## Rejected requests and `TraceLayer`

```rust
Router::new()
    .route(...)
    .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
    .layer(tower_http::trace::TraceLayer::new_for_http())
```

`tower`'s `.layer(...)` calls wrap outside-in in the order they're written,
which means the *last* `.layer(...)` call ends up as the *outermost* layer
around the whole stack. `TraceLayer` is added last, so it wraps
everything — including `rate_limit_middleware` — meaning every request,
rejected or not, passes through `TraceLayer` first on the way in and last
on the way out. A 429 rejection still gets `TraceLayer`'s generic
method/path/status/latency log line; `rate_limit_middleware`'s own
`tracing::warn!` is additional detail *inside* that. If the order were
reversed (`TraceLayer` added first, `rate_limit_middleware` last), the rate
limiter would sit outside `TraceLayer`, and a rejected request would short
the middleware stack before ever reaching `TraceLayer` — no trace at all
for exactly the requests you'd most want visibility into.

## Why `f64`, concretely

With `refill_rate = 0.5` and `tokens: u32`, after 1 second `elapsed *
refill_rate = 0.5`, and `u32` has no fractional part — that `0.5` gets
truncated to `0` on assignment (or requires an explicit, lossy cast),
so the bucket gains *nothing* after 1 second. After 2 seconds, the exact
value would be `1.0`, but if you'd been truncating and discarding the
fractional accumulation on every intermediate call rather than one
lump-sum calculation, you could easily end up perpetually stuck at `0`
tokens gained no matter how much time passes, depending on how often
`refill` gets called. With `tokens: f64`, `elapsed * refill_rate` accumulates
exactly: `0.5` after 1 second, `1.0` after 2 seconds — sub-token progress
is never lost between calls, which is exactly why `token_bucket_refills_over_time_up_to_capacity`
can assert a precise, reliable refill after exactly 1 second at a 1
token/sec rate.

## What breaks running 3 replicas behind a load balancer

Both the cache and the rate limiter break, for the same underlying reason:
both live in one process's memory (`Mutex<HashMap<...>>` and `Mutex<TokenBucket>`
respectively), and a load balancer sends different requests to different
replicas with no guarantee the same replica ever sees the same client or
title twice. Three replicas means three independent, silently-diverging
caches (a title cached on replica A is still a cold miss on replica B — the
capacity benefit of caching shrinks roughly by a factor of however many
replicas exist) and three independent rate-limit buckets (a client capped
at "10 requests/second" can actually get 30/second by unluckily spreading
requests across all three replicas, defeating the limit's whole purpose).
The fix for both is the same one covered in `phase4-backend-advanced/01-caching-with-redis`:
move shared, cross-replica state out of process memory and into something
every replica talks to — Redis for the cache (exactly what `RedisCache`
in that lesson demonstrates), and Redis (or another shared store) for the
rate limiter's token counts too, so "10 requests/second" means 10 total
across every replica, not 10 per replica.
