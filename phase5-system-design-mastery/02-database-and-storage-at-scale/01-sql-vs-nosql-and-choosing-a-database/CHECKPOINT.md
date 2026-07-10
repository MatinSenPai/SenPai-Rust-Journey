# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can reason about a real access pattern,
not recite a definition.

1. Name the four data models usually lumped under "NoSQL" and, for each
   one, name one system from this lesson and one workload it's genuinely
   well-suited to. Why is "NoSQL vs SQL" a misleading way to frame a
   database choice?

2. `capstone-taskforge/taskforge-core/src/job.rs`'s `JobStatus` enum and
   `docs/adr/0003-job-state-machine.md` both insist a job can never be
   observable in a nonsensical combination of states. Explain, specifically,
   why a single-writer relational database with `FOR UPDATE SKIP LOCKED`
   makes that guarantee easier to uphold than an eventually-consistent,
   AP-leaning wide-column store would. What could go wrong with the
   AP-leaning version, concretely?

3. Walk through the five-question decision framework in this lesson using
   `phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`'s
   `anime` table as the example. Which question(s) point toward relational,
   and which (if any) are irrelevant for this specific table?

4. Pick one of the four NoSQL models from this lesson and describe a
   *hypothetical* new feature for TaskForge (not something built in this
   repo) where that model would genuinely be a better fit than Postgres.
   Be specific about which property of that model is doing the work (schema
   flexibility, key-value speed, horizontal write scale, graph traversal).

5. A teammate says "we should migrate off Postgres to MongoDB because
   NoSQL scales better." Using this lesson's vocabulary, what follow-up
   question would you ask them before agreeing or disagreeing, and why does
   the answer matter more than the general "NoSQL scales better" claim?
