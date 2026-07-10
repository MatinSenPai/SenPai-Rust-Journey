# Solution

```rust
pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}
```

Two lines of real logic, but the interesting part is the `PgPoolOptions`
configuration, not the bodies.

## What happens to queries 6-20

Nothing dramatic: `sqlx::Pool` maintains an internal queue. When all 5
connections are checked out, a 6th caller's `.fetch_one(&pool)` (or
whatever query method) simply `.await`s until a connection is returned to
the pool — by whichever of the first 5 finishes first — at which point the
waiting caller is woken and gets that connection. No query is rejected or
dropped; they just queue, invisibly, behind the `.await` point. This is
exactly why the test asserts all 20 results come back correct rather than,
say, expecting some to error — from the caller's point of view, a pool
under contention just gets *slower*, not *less correct*. `acquire_timeout`
is the only thing standing between "queues, invisibly" and "queues
forever": if a caller waits longer than 3 seconds for a free connection, it
gets an error instead of hanging indefinitely.

## Why eager `.connect()` matters

`connects_and_passes_a_health_check` and
`a_bad_url_fails_to_connect_instead_of_hanging` both rely on this: because
`PgPoolOptions::connect()` eagerly opens a real connection before
returning, a bad URL surfaces as an `Err` from `connect_pool` itself. If
`sqlx` instead lazily deferred connecting until the first query, a bad
`database_url` would silently produce an apparently-valid `PgPool` value,
and the error would only appear later, on whatever unlucky query happened
to run first — a much worse debugging experience, and a test that couldn't
distinguish "bad URL" from "any other runtime failure" without actually
issuing a query.

## The real ceiling on `max_connections`

Raising it from 5 to 500 doesn't scale the amount of real concurrency you
can extract from a single Postgres server — it just changes how many of
*this process's* real connections are open at once. Postgres itself has its
own server-side `max_connections` setting (100 by default), and every
authenticated connection consumes real server memory even when idle. In a
production deployment running multiple replicas of this same service, each
with `max_connections(500)`, you can very easily exhaust Postgres's own
connection ceiling long before any single replica's pool is full — the
practical fix in real systems at that scale is usually a connection
pooler in front of Postgres itself (PgBouncer, or a managed equivalent),
not raising each app instance's pool size indefinitely.

## What breaks without `acquire_timeout`

Nothing changes in `many_concurrent_queries_share_a_small_pool_safely` —
five connections easily handle 20 fast local queries well within any
timeout. The test that would change is
`a_bad_url_fails_to_connect_instead_of_hanging`: without
`acquire_timeout`, `sqlx` falls back to its own internal default (30
seconds against an address that never actively refuses the connection,
as observed empirically while building this lesson), so the test would
still eventually pass but take 10x longer for no benefit — a concrete,
measurable cost of skipping this setting, not just a theoretical one.

## `pool.clone()` per task, and where that pattern first showed up

`tokio::spawn` requires its future to be `'static` — it may outlive the
scope that spawned it, so it can't borrow anything with a shorter
lifetime, including a stack-local `&PgPool`. Each spawned task instead gets
an owned `PgPool` via `.clone()`, which is cheap because `PgPool` wraps an
`Arc` around its actual connection-management state — cloning bumps a
reference count, it doesn't duplicate any connections. This is the exact
same shape as Phase 2's threads-and-`Arc`-`Mutex` lesson (`Arc::clone` per
thread so each thread owns a handle to the same shared state) and
`capstone-taskforge`'s `Arc<dyn JobStore>` passed into every worker task —
`Arc`-wrapped shared state, cloned per task/thread rather than borrowed, is
the standard way to hand many concurrent tasks access to one underlying
resource in Rust.
