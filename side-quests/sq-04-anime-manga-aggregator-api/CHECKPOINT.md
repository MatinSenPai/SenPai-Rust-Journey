# Checkpoint

1. `get_or_fetch_never_calls_compute_on_a_hit` asserts a counter stays at
   `0` on a cache hit. Why does `compute` need to be a generic `F: FnOnce()
   -> Fut` parameter — a closure returning a future — rather than just an
   already-`await`ed `Option<TitleInfo>` value passed in directly? What
   would change (concretely, in terms of when work actually happens) if
   `get_or_fetch` instead took `compute: Option<TitleInfo>`?
2. `TtlCache` stores `Arc<dyn Clock>` rather than a concrete `SystemClock`
   or `FakeClock` type directly. What would break about testing TTL
   expiry if `TtlCache` only ever used `Instant::now()` directly instead?
3. `rate_limit_middleware` is `axum` middleware that runs *before* every
   handler, wrapping the router with `.layer(...)`. Trace through what
   happens to a rejected (429) request with respect to `TraceLayer` — does
   a rejected request still get logged by `TraceLayer`, given the order
   `.layer(...)` calls appear in `app()`? Why does that ordering matter?
4. `TokenBucket::refill` uses `f64` for `tokens` and `capacity` instead of
   an integer type. Walk through what would go wrong with a `refill_rate`
   of `0.5` tokens/second if `tokens` were a `u32` instead — be concrete
   about what value `tokens` would hold after 1 second vs. after 2 seconds
   in each version.
5. Everything in this project is a single in-memory process. If you needed
   to run 3 replicas of this service behind a load balancer, what would
   break first — the cache, the rate limiter, both, or neither — and what
   earlier lesson in this curriculum covered the fix for each?
