# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain the actual tradeoffs, not
whether a test passes.

1. Two Postgres shards, each with their own `BIGSERIAL` sequence, both
   hand out `id=501` to different rows. Explain exactly why this happens —
   what assumption does `BIGSERIAL` rely on that sharding breaks?
2. `taskforge-core`'s `JobId` wraps a `Uuid`, generated with
   `Uuid::new_v4()`. Explain specifically why `taskforge-api` being run as
   multiple replicas (the horizontal-scaling setup from the CAP/scaling
   lesson) makes a coordination-free ID scheme like this one necessary
   rather than just convenient.
3. Explain the B-tree index-locality cost of random UUIDs in your own
   words: what property does `BIGSERIAL` give you for free that
   `Uuid::new_v4()` doesn't, and what does that cost in practice at high
   insert volume?
4. Describe the three parts of a Snowflake-style 64-bit ID and explain how
   the bit layout is what lets these IDs be both coordination-free at
   generation time *and* roughly sortable by creation time — a property
   neither `BIGSERIAL` alone nor random UUIDs alone give you together.
5. Snowflake IDs still need "coordination" — just less of it than a shared
   sequence. What specifically needs to be coordinated, how often, and why
   is that a smaller cost than a `BIGSERIAL` sequence's per-insert
   coordination?
6. Explain why `phase3-backend-foundations/04-postgres-and-sqlx`'s
   anime-catalog lesson and `capstone-taskforge` made different ID
   choices, in terms of the actual constraint each system has (or doesn't
   have) — not "UUIDs are more modern."
