# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can trace the actual mechanism, not
whether a test passes.

1. `taskforge-worker/src/backoff.rs`'s `compute_backoff` is called from two
   different failure paths in `pool.rs`'s `run_one_job`. Name both, and
   explain why a handler *panicking* is treated the same way as a handler
   returning `Err(JobError)` for backoff purposes.
2. Explain, specifically, why jitter is necessary in addition to
   exponential backoff — what failure pattern does pure (non-jittered)
   exponential backoff still allow, and why does adding randomness fix it?
3. Walk through the worked payment-service example in this lesson without
   and with a circuit breaker. What's different about what happens to
   `taskforge-worker`'s other, unrelated job types in each version?
4. `acquire_timeout(Duration::from_secs(3))` in
   `phase3-backend-foundations/04-postgres-and-sqlx/01-connecting-and-pooling`
   is a timeout on acquiring a pooled connection, not on the query itself.
   Explain what specifically goes wrong if it's omitted, and under what
   load pattern you'd actually notice the bug (i.e., what would have to be
   true for a missing `acquire_timeout` to actually bite).
5. Describe, concretely, how you'd apply the bulkhead pattern to
   `taskforge-worker` if it handled both payment jobs and email-sending
   jobs through the same worker pool. What specific change would you make,
   and what failure does it prevent that a circuit breaker alone would not?
6. A circuit breaker and a bulkhead solve related but different problems.
   In one or two sentences each, state what problem each one solves, and
   why you generally want both rather than treating either as a
   substitute for the other.
