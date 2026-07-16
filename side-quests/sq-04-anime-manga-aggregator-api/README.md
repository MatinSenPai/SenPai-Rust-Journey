# Side-quest 4 — Anime/Manga Aggregator API

The warm-up before the capstone: three Phase 4 ideas — cache-aside caching,
token-bucket rate limiting, and `tracing`-based observability — combined
into one small, genuinely fun project instead of three separate exercises.
Same consolidation spirit as every side-quest: no new *concepts* here, just
putting recent ones to work together, the way they'd actually coexist in
one real service.

## What it does

An API aggregating info about anime/manga titles (title, synopsis,
episode/chapter count). In production this would call a real upstream
source — the [Jikan API](https://jikan.moe) (an unofficial MyAnimeList
API) via `reqwest` is the natural choice — but this side-quest simulates
that with an in-memory seed dataset and an artificial delay
([`fetch_title_from_source`]), so the project stays dependency-free and
fast to run while still teaching the exact shape of a real
"expensive-lookup-behind-a-cache" service.

```
GET /titles       every known title, no caching involved
GET /titles/{id}  one title — cache-aside-fronted, logs a hit or miss
```

## The three pieces

1. **Cache-aside** (`TtlCache` + `get_or_fetch`) — check the cache first;
   on a hit, return immediately, the "expensive" lookup never runs at all.
   On a miss, run the lookup, cache the result, return it. Exactly the
   pattern from `phase4-backend-advanced/01-caching-with-redis/01-cache-aside-ttl-invalidation`,
   just backed by a `Mutex<HashMap<...>>` here instead of Redis — swapping
   in a real `redis`-backed cache later would only mean writing a new type
   with the same `get`/`set` shape, nothing in `get_or_fetch` or the axum
   handlers would need to change.
2. **Rate limiting** (`TokenBucket` + `rate_limit_middleware`) — every
   request must acquire a token from a shared bucket or gets rejected with
   `429` before it reaches a handler. Same algorithm as
   `phase4-backend-advanced/02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit`,
   hand-rolled as `axum` middleware here so you can see exactly what
   `tower::limit::RateLimitLayer` does for you under the hood.
3. **Observability** (`tracing::info!`/`warn!` calls already wired into
   `get_title` and the rate limiter) — every request logs its method,
   path, cache hit/miss, and latency as structured fields, the same
   discipline as `phase4-backend-advanced/05-observability/01-structured-logging-with-tracing`.

## What's already done vs. your task

Everything except four functions is given, fully working: the seed data,
`fetch_title_from_source`, the `Clock`/`FakeClock` time-injection setup (so
tests control TTL expiry without ever sleeping for real), both axum
handlers, the rate-limiting middleware, and `app()`'s routing/layering.
Four `todo!()`s are the actual exercise:

- `TtlCache::get` / `TtlCache::set` — the cache itself.
- `get_or_fetch` — the cache-aside orchestration: check cache, compute on
  miss, cache the result, report hit-or-miss to the caller.
- `TokenBucket::refill` — the token bucket's refill math (reused/adapted
  from the rate-limiting lesson — same formula, same reasoning about why
  `tokens` is `f64` and `now` is an explicit parameter for testability).

## Your task

Open `src/lib.rs`. Implement `TtlCache::get`, `TtlCache::set`,
`get_or_fetch`, and `TokenBucket::refill`.

## Checkpoint

`cargo test -p sq-04-anime-manga-aggregator-api`. Then `CHECKPOINT.md`,
then `solution/SOLUTION.md`.

## Stretch extension (optional, no solution provided)

### Cover-image upload

Every real catalog API eventually grows a "please stop sending me JSON"
endpoint: file upload. Add one — it's the only multipart handling in the
whole curriculum, and it's spec-only on purpose: you design the shape.

- `POST /titles/{id}/cover` accepting `multipart/form-data` — enable the
  `multipart` feature on the workspace's existing `axum` dependency and use
  the `Multipart` extractor.
- `GET /titles/{id}/cover` serves the image back with the right
  `Content-Type`.
- The rules that make it safe, each one a test:
  - **Size cap** — reject bodies over ~2 MB (`DefaultBodyLimit` layer)
    with `413 Payload Too Large`.
  - **Type allowlist** — only `image/png` / `image/jpeg`; anything else is
    `415 Unsupported Media Type`. Trust the bytes, not the filename.
  - **Never trust the client's filename** — store as
    `uploads/{title_id}.{ext}` (or a `uuid`), *never* interpolate the
    uploaded filename into a path. (`../../etc/passwd.png` is the classic
    reason why — path traversal.)
  - `404` if the title doesn't exist, on both endpoints.
- **Acceptance check:**
  `curl -F "cover=@some.png" localhost:3000/titles/1/cover` succeeds, the
  file appears under `uploads/`, the GET round-trips it, and an 11 MB file
  or a `.exe` upload gets cleanly rejected — no panic, no partial file left
  behind.
