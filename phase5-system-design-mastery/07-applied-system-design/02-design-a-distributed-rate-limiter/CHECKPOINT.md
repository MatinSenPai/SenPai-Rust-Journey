# Checkpoint

1. `concurrent_callers_never_exceed_the_limit` fires 20 concurrent
   `check()` calls at a limit of 5 and asserts exactly 5 succeed. What
   specifically about the Lua script's execution model makes that
   guarantee hold, even though 20 separate Rust tasks are genuinely racing
   against the same Redis key at nearly the same instant?
2. Why does `check` use a Lua script instead of calling `.incr()` then
   `.expire()` as two separate commands from Rust? Walk through the
   specific failure this lesson's README describes, concretely: what state
   would the key be left in, and what would that mean for the caller it
   belongs to?
3. This lesson's tests don't use `#[serial(...)]`, unlike every
   Postgres-backed multi-test lesson elsewhere in this repo. What property
   of Redis keys (versus a shared Postgres table) makes that safe here?
   Could you have used the same "give every test its own key/table"
   trick for the Postgres lessons instead of `#[serial]` — what would
   that have cost you that a Redis key doesn't?
4. `the_window_resets_after_it_expires` sleeps for slightly over 1 second
   in a test with a 1-second window. What would happen to this test if the
   window were, say, 60 seconds instead — would you keep the same
   `tokio::time::sleep`-based approach, or is there a better pattern? (You
   don't need to implement it — describe what you'd reach for.)
5. Compare this lesson's fixed-window algorithm to the in-process
   `TokenBucket` from `phase4-backend-advanced/02-rate-limiting-and-backpressure`.
   A fixed window has a known edge-case burst problem at window
   boundaries (a caller could use their whole limit in the last second of
   one window, then their whole limit again in the first second of the
   next — 2x the intended rate in a 2-second span). Does the token bucket
   algorithm have the same problem? Why or why not?
