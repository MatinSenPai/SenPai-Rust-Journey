# 07.2 — Design a distributed rate limiter

`phase4-backend-advanced/02-rate-limiting-and-backpressure` already built a
token bucket rate limiter, and side-quest 4's aggregator API reused the
same algorithm as `axum` middleware. Both work — inside **one process**.
This lesson closes the gap both of those lessons' own checkpoints already
named: run 3 replicas of either behind a load balancer, and a client can
get 3x the intended limit just by having requests spread across
replicas, since each replica's `TokenBucket` has no idea the other two
exist.

## Why Redis, and why a fixed window instead of a token bucket

The fix is the same one this repo keeps reaching for: move the mutable
state that needs to be *shared* out of process memory and into something
every replica can reach — the identical move the caching lesson made for
its cache, just applied to a rate limiter's counters instead. This lesson
uses a simpler algorithm than the in-process lessons' token bucket: a
**fixed window counter** — "at most `limit` calls per `key` per `window`"
— because it maps onto exactly one Redis command pattern (`INCR` +
`EXPIRE`) with no per-request math to keep synchronized across replicas.
(A distributed token bucket is also possible — Redis's `INCRBYFLOAT` plus
a timestamp-tracking key can do it — but the fixed window is the more
common real-world default specifically because it's simpler to reason
about and implement correctly under concurrency, which is the whole point
of this lesson.)

## The one command that has to be atomic

```lua
local current = redis.call('INCR', KEYS[1])
if current == 1 then
    redis.call('EXPIRE', KEYS[1], ARGV[1])
end
return current
```

Why not just call `.incr()` then `.expire()` as two separate Rust-level
Redis commands? Because between those two commands, another concurrent
caller could see the freshly-`INCR`'d key with *no* expiry set yet — and
if the process crashed or the connection dropped in that exact gap, the
key would live forever, permanently rate-limiting that caller at 0
remaining calls. Wrapping both commands in a `redis::Script` sends them to
Redis as one atomic unit — Redis executes an entire Lua script without
interleaving any other client's commands in the middle, the same
"no gap for another caller to land in" guarantee Postgres's `UPDATE ...
RETURNING` gave the URL shortener lesson, just enforced by Redis's
single-threaded command execution instead of a row lock.

## Test isolation without `#[serial]`

Every Postgres-backed lesson elsewhere in this repo that has multiple
tests sharing one mutable table needs `#[serial(...)]` to stay
deterministic under `cargo test`'s default parallel execution (see
`phase3-backend-foundations/04-postgres-and-sqlx/02-migrations/README.md`
for the real bug that taught this repo that lesson). This lesson's tests
don't need it: Redis keys are free and namespaced per test
(`unique_key("test_name")` appends a nanosecond timestamp), so no two
tests, even running concurrently, ever touch the same key. When the
shared backend makes per-test isolation this cheap, prefer it over
serializing — it's what actually lets your test suite exploit `cargo
test`'s default concurrency instead of fighting it.

## Your task

Open `src/lib.rs`. Implement `RedisRateLimiter::check`.

## Checkpoint

With a live Redis (`redis-server --daemonize yes` if not already running):

```sh
cargo test -p p5-07-02-design-a-distributed-rate-limiter -- --ignored
```

Then `CHECKPOINT.md`, then `solution/SOLUTION.md`.
