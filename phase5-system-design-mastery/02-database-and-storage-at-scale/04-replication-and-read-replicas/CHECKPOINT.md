# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can reason about lag and its fixes with a
concrete scenario, not recite definitions.

1. Explain synchronous vs. asynchronous replication as a latency-vs-
   consistency tradeoff, using a concrete "what could go wrong" example for
   each — one where synchronous replication hurts you, one where
   asynchronous replication hurts you.

2. Read replicas scale reads, sharding scales both reads and writes.
   Explain why read replicas don't help write throughput at all, using
   what you know about leader-follower replication's write path.

3. Walk through the read-your-writes worked example in this lesson (post a
   comment, refresh, don't see it) in your own words, using TaskForge
   instead of a comment system: describe a concrete sequence of API calls
   against `taskforge-api`'s handlers where a client could enqueue a job
   and then, if reads were served from a lagging replica, fail to see it
   in a subsequent `GET`.

4. Compare the "read-from-primary-after-write" fix and the "session-sticky
   routing" fix. Which one gives a stronger guarantee, and what does the
   weaker one give up in exchange for spreading load more evenly?

5. This lesson describes two ways to add read replicas to TaskForge:
   splitting `JobStore` into reader/writer traits, or hiding the routing
   inside one `ReplicatedPostgresJobStore`. Pick one and argue for it —
   what's the concrete advantage of your choice, and what would a
   contributor lose by not being able to tell, in `taskforge-api/src/handlers.rs`,
   which path (primary or replica) a given `state.store` call takes?

6. `taskforge-api`'s handlers currently all depend on one
   `Arc<dyn JobStore>`. Why does that trait boundary (rather than, say,
   `taskforge-api` calling `sqlx` directly) make adding read replicas an
   additive change instead of a rewrite? Point to the specific ADR that
   already established this pattern for a different reason.
