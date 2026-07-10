# Solution

```rust
pub async fn check(&self, key: &str) -> Result<bool, RateLimiterError> {
    let script = redis::Script::new(
        r"
        local current = redis.call('INCR', KEYS[1])
        if current == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[1])
        end
        return current
        ",
    );

    let mut conn = self.connection().await?;
    let count: i64 = script
        .key(key)
        .arg(self.window.as_secs())
        .invoke_async(&mut conn)
        .await
        .map_err(|e| RateLimiterError::Backend(e.to_string()))?;

    Ok(count <= self.limit as i64)
}
```

## Why the Lua script's execution genuinely guarantees no over-count

Redis processes commands on a single thread — including the body of a
Lua script submitted via `EVAL`/`EVALSHA` (which is what `redis::Script`
compiles down to). Once Redis starts executing a script, it runs the
script's `INCR` and (conditionally) `EXPIRE` to completion before
processing *any* other client's command, including another concurrent
`check()` call's own script invocation. So even though 20 Rust tasks in
`concurrent_callers_never_exceed_the_limit` all call `.check()` at
essentially the same wall-clock instant, Redis still serializes their
script executions internally — the 6th script to actually run inside
Redis sees `current == 6` (already over the limit of 5) no matter which
of the 20 Rust tasks "won the race" to get there first. The concurrency
happens on the Rust/network side; Redis's own execution is never
concurrent with itself.

## What a non-atomic `.incr()` then `.expire()` would risk

Consider the sequence: caller A calls `.incr()`, gets back `1` (a brand
new key). Before caller A's *next* line calls `.expire()`, the process
crashes, or the connection to Redis drops, or the task simply gets
descheduled for an unlucky moment. The key now sits in Redis with a count
of `1` and **no TTL at all** — it never expires. Every future call under
that same key sees the count keep climbing (2, 3, 4, ...) and eventually
permanently exceeds `limit`, with no window ever resetting, because
nothing ever set the expiry that was supposed to reset it. That caller is
rate-limited forever, for a bug that has nothing to do with them actually
calling too often. Wrapping both commands in one Lua script closes this
gap entirely — `EXPIRE` either runs as part of the same atomic unit as the
`INCR` that created the key, or the whole script doesn't run at all (e.g.
a connection failure before the script starts) and nothing was ever
half-applied.

## Why no `#[serial]` here, and whether the trick transfers to Postgres

Redis keys are essentially free — creating a million distinct, unrelated
keys costs nothing beyond the memory each one's value occupies, and there
is no schema, no `CREATE TABLE`, no migration to run per key. That's what
makes `unique_key("test_name")` cheap: every test effectively gets its own
private slice of the keyspace with zero setup cost. The same trick
*could* apply to Postgres in principle — give every test its own table
name, or even its own schema — but the cost is real: every test would
need to either run its own `CREATE TABLE`/DDL first (slower, and now
you're testing your migration/schema-creation code path on every single
test instead of once) or you'd need a pre-provisioned pool of tables to
round-robin through (real setup complexity for what `#[serial]` solves in
one line). `testcontainers`
(`phase3-backend-foundations/07-error-handling-and-testing-at-scale/02-integration-tests-with-testcontainers`)
is actually the "give every test its own everything" answer for
Postgres — a whole fresh database per test — but it trades that isolation
for the cost of booting a real container per test, which is why this
repo reaches for `#[serial]` on a shared database as the lighter-weight
default and reserves `testcontainers` for the lesson specifically about
that heavier isolation technique.

## Fixed windows at longer timescales

For a 60-second window, sleeping 61 real seconds in a test would work but
would make the test suite slow for no good reason. The pattern this repo
already uses for exactly this problem is a `Clock` trait — a real
`SystemClock` for production and a `FakeClock` tests can `.advance(...)`
on demand — see `phase4-backend-advanced/01-caching-with-redis`'s and side-quest
4's `TtlCache`, both of which take this approach specifically so TTL/window
expiry tests never need to actually wait out the real duration. This
lesson's `RedisRateLimiter` doesn't have that seam today (Redis's own
`EXPIRE` uses Redis server time, not anything the Rust process controls) —
extending it to support injectable time would mean not relying on
Redis-native TTL at all, and instead storing an explicit window-start
timestamp per key that the Lua script compares against an injected "now"
argument, checking/rotating the window in Lua rather than leaning on
Redis's own expiry.

## Does the token bucket have the same boundary-burst problem?

No — and that's the real algorithmic advantage a token bucket has over a
fixed window, not just a stylistic difference. A fixed window resets
*abruptly*: at the exact moment one window ends and the next begins, the
counter snaps back to zero, so a caller can legitimately burst their full
limit in the last instant of one window and their full limit again in the
first instant of the next, clustering 2x the intended rate into a short
span around the boundary. A token bucket refills *continuously* — see
`TokenBucket::refill`'s `elapsed * refill_rate` math from the
rate-limiting lesson — there's no single instant where a large batch of
new capacity appears all at once; tokens accrue smoothly over time, so
there's no boundary to burst around in the first place. This is the
concrete tradeoff this lesson's README gestured at: the fixed window
traded a stronger smoothness guarantee for a much simpler, more easily
made-atomic-in-Redis implementation.
