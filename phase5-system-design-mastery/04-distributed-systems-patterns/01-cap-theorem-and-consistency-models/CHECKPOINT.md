# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea precisely, not
whether a test passes.

1. Define linearizability in your own words, then explain why "eventually
   consistent" systems don't satisfy it even though they're commonly
   described as "consistent" in casual conversation.
2. `taskforge-storage`'s `PostgresJobStore` provides strong consistency
   (linearizability). Walk through the specific correctness bug that would
   become possible if it instead provided only eventual consistency, like
   DynamoDB or Cassandra do by default — be specific about which two
   workers, doing what, at what moment, produce the bad outcome.
3. Explain the difference between causal consistency and read-your-writes
   consistency using the "comment and reply" example and the "update your
   own bio" example. Why is read-your-writes the cheaper guarantee to
   provide of the two?
4. Strong consistency implies causal consistency implies read-your-writes
   consistency, but plain eventual consistency doesn't fit that chain at
   all. Explain why, in terms of what ordering guarantee (if any) each
   model makes.
5. Pick a system that's a good fit for eventual consistency and one that
   isn't (you can reuse `taskforge-storage` for the second one, or pick a
   different example from earlier in this repo). For each, explain what
   about the *cost of a stale read* drives the choice.
