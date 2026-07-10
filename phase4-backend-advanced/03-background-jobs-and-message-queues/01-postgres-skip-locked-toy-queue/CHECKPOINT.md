# Checkpoint

1. Read `taskforge-storage/src/postgres.rs`'s `claim_next` side by side with
   this lesson's `PostgresQueue::claim_next`. List every structural
   similarity you see between the two queries (the CTE, the lock clause,
   the ordering, the `RETURNING`). What's different, and why (hint: think
   about what real job state — retries, `next_attempt_at` — the capstone
   needs that this toy doesn't)?
2. `InMemoryQueue` uses a plain `std::sync::Mutex`, not `tokio::sync::Mutex`.
   Both compile here. `claim_next`'s critical section (pop from `pending`,
   insert into `in_progress`) never `.await`s anything while the lock is
   held. Why does that fact make a `std::sync::Mutex` the right (and
   slightly cheaper) choice over `tokio::sync::Mutex` here? What would go
   wrong if the critical section *did* need to `.await` something?
3. `concurrent_claims_never_double_claim_a_job` proves the "no double claim"
   property using a `Mutex` inside one process. Why can't the same
   `std::sync::Mutex`-based approach give you that guarantee if
   `InMemoryQueue` were, instead, shared across two separate OS processes
   (e.g. two separately-run `cargo run` binaries)? What does Postgres (or
   any real database/broker) provide that a mutex living in one process's
   memory fundamentally cannot?
4. `complete` fails with `QueueError::NotClaimed` both for a job that was
   never claimed and for a job that was already completed once. Why is
   reusing the same error variant for both cases the right call here,
   rather than adding a separate `AlreadyCompleted` variant?
5. Skim `capstone-taskforge/docs/adr/0002-postgres-backed-queue.md`'s
   "Consequences" section. In your own words, what's the throughput
   ceiling argument for *why* a real message broker (module 02's topic)
   eventually becomes the better choice over a Postgres-backed queue, and
   why doesn't that trade-off matter yet for TaskForge's v1?
