# Solution

## `TokenBucket::new`

```rust
pub fn new(capacity: u32, refill_rate: f64, now: Instant) -> Self {
    Self {
        capacity: capacity as f64,
        refill_rate,
        tokens: capacity as f64,
        last_refill: now,
    }
}
```

Starts full — `tokens == capacity` — so the very first burst of requests up
to `capacity` succeeds immediately, matching the README's "bursts are
allowed, up to `capacity`" property.

## `refill`

```rust
fn refill(&mut self, now: Instant) {
    let elapsed = now.saturating_duration_since(self.last_refill).as_secs_f64();
    self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
    self.last_refill = now;
}
```

`saturating_duration_since` (rather than plain subtraction) protects against
a `now` that's earlier than `last_refill` — shouldn't happen in this
lesson's tests (time only moves forward), but is the same defensive habit
you'd want in production code where `now` might come from a source you don't
fully control. `.min(self.capacity)` is the "never overflow" cap. Note
`last_refill` is set to `now` unconditionally, every call — see checkpoint
question 3.

## `try_acquire`

```rust
pub fn try_acquire(&mut self, now: Instant) -> bool {
    self.refill(now);
    if self.tokens >= 1.0 {
        self.tokens -= 1.0;
        true
    } else {
        false
    }
}
```

Refill first, then decide — this ordering matters: a call that arrives after
enough time has passed to earn a fresh token must see that token *before*
the accept/reject check, or a bucket that's been idle for exactly long
enough to refill one token would incorrectly reject the very request that
should now be allowed.

## Checkpoint answers (worked)

**1. Why `now: Instant` as a parameter?** Calling `Instant::now()` inside
`try_acquire` would make every test that wants to assert "no tokens have
refilled yet" or "exactly 1 second has passed" either flaky (real
`sleep`-based timing has jitter) or slow (waiting out real seconds per
test). Passing `now` explicitly makes elapsed time a value the test fully
controls — `now + Duration::from_secs(1)` is exact and instantaneous, no
different in principle from the caching lesson's `FakeClock`.

**2. Why `f64` for `tokens`?** With `refill_rate = 2.0` and a 100ms step, each
call earns `0.2` tokens. If `tokens` were an integer, that `0.2` would need
either truncation (losing sub-token progress on every single call — after 50
calls at 100ms/2 tokens-per-sec you'd have lost almost all the fractional
accumulation, undercounting badly) or a separate fixed-point/remainder
scheme to avoid the loss. `f64` sidesteps this entirely: fractional progress
just accumulates naturally across calls, which is exactly what
`sustained_rate_matches_refill_rate_over_a_long_window` depends on to land
in the expected `9..=12` range.

**3. Why update `last_refill` even when zero tokens were added?** Because
`last_refill`'s job is "the last instant we accounted for," not "the last
instant we successfully added a token." If it only updated on a nonzero
gain, two calls a microsecond apart would both compute `elapsed` from the
*same* stale `last_refill` on a third call much later, but worse: elapsed
time between the first and second call would never be counted at all
(dropped on the floor), silently under-crediting the bucket. Updating
unconditionally guarantees elapsed time is accounted for exactly once, no
gaps and no double-counting.

**4. Token bucket vs. cache TTL.** Both store a timestamp and compare it to
"now," but they're protecting opposite things. A cache TTL protects against
staleness — it answers "is this stored *value* still trustworthy?" and, once
expired, the data is simply gone (recomputed fresh). A token bucket protects
against *rate* — it answers "has too much *work* happened too recently?" and
the "value" (the token count) is deliberately allowed to partially recover
over time rather than being all-or-nothing.

**5. Which backpressure strategy, and what `async fn acquire` would need.**
`try_acquire`'s `bool` return is strategy 1, "reject immediately" — the
caller finds out synchronously and is responsible for deciding what to do
(retry later, return `429`, drop the request). An `async fn acquire(&mut self)`
that *waits* for a token instead would need: (a) some way to be woken up
when a token becomes available rather than busy-polling — either compute
"how long until at least one token exists" (`(1.0 - self.tokens) / self.refill_rate`
seconds) and `tokio::time::sleep` that long before rechecking, or register
with a `Notify`/waker that a companion background task pings on a timer; (b)
a decision about what happens if many callers are all waiting at once —
FIFO fairness isn't automatic just from adding `.await`, you'd likely want
an explicit queue (this is exactly strategy 2, "queue with a bound," and is
close to what `tower::limit::RateLimitLayer` actually implements via
`tower::buffer`).
