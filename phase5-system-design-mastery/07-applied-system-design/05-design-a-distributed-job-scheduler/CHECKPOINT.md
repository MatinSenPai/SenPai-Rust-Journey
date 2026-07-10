# Checkpoint

Answer these by actually opening and re-reading the named files — this
lesson is explicitly about looking at your own code with fresh eyes, not
recalling it from memory.

1. Open `capstone-taskforge/taskforge-storage/src/postgres.rs`'s
   `claim_next`. In your own words, connect its `FOR UPDATE SKIP LOCKED`
   query directly to Module 4's `02-idempotency-and-distributed-locking`
   lesson's three-way comparison (Postgres row lock vs. Redis SETNX/TTL
   vs. Zookeeper/etcd). Why was a Postgres row lock the right call for
   TaskForge specifically, given it already needed Postgres for job
   storage anyway?
2. Open `capstone-taskforge/docs/adr/0004-worker-failure-handling.md`.
   It documents a known, shipped gap: no automatic reclaim-on-crash.
   Describe, concretely, the exact sequence of events that would cause a
   real job to get permanently stuck in `Running` status, never picked up
   by any worker again. Then describe the shape of the fix (a sweep query,
   a timeout column, what it would need to check) — you don't need to
   write the code, just the design.
3. `taskforge-core`'s job ids are `Uuid`s, not an auto-increment integer.
   Trace this decision back to Module 4's `04-unique-id-generation`
   lesson: what's the actual failure mode a `BIGSERIAL` id would have
   caused for `taskforge-api`, specifically, that `Uuid::new_v4()` avoids?
4. If TaskForge needed to add an idempotency key to `POST /jobs` (Module
   4's idempotency lesson's sketch), walk through exactly what would need
   to change in `taskforge-core`, `taskforge-storage`, and
   `taskforge-api` — new columns, a new check before enqueueing, what the
   response looks like on a duplicate submission.
5. Pick one system design question from Module 7 you haven't built
   (chat, notifications, URL shortener, rate limiter — whichever you
   built, pick a different one) and describe, in a paragraph, how you'd
   adapt TaskForge's actual architecture (not a hypothetical new system)
   to solve it. What would you reuse directly, and what would need to be
   genuinely different?
