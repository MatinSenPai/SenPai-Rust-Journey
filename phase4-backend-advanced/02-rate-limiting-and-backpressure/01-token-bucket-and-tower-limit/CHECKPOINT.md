# Checkpoint

1. `try_acquire` takes `now: Instant` as an explicit parameter instead of
   calling `Instant::now()` internally. What would you lose (specifically,
   about the test suite) if it called `Instant::now()` itself instead?
2. `tokens` is stored as `f64`, not `u32`. Walk through
   `never_exceeds_capacity_even_after_a_long_idle_period` and
   `sustained_rate_matches_refill_rate_over_a_long_window` — what would go
   wrong (or become harder to write correctly) if `tokens` were an integer
   type instead?
3. `refill` updates `self.last_refill = now` even on a call where zero
   tokens ended up being added (e.g. two calls a microsecond apart). Why is
   that still correct — what would happen if `last_refill` were only
   updated when at least one whole token was added?
4. Compare this lesson's token bucket to `phase4-backend-advanced/01-caching-with-redis`'s
   TTL mechanism. Both involve a stored timestamp and an elapsed-time
   calculation against "now." What's fundamentally different about what
   each one is protecting?
5. Of the three backpressure strategies in the README (reject immediately,
   bounded queue, unbounded queue), which one does `try_acquire`'s `bool`
   return type implement? Sketch (in words, no code needed) what changing
   `try_acquire` to `async fn acquire(&mut self)` that *waits* for a token
   instead of immediately returning `false` would require.
