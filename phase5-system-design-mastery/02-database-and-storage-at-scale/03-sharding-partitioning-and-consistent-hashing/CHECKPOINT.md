# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain the mechanism, not recall a
definition.

1. Explain the difference between partitioning and sharding in one or two
   sentences each, and give an example of a system that could reasonably
   use partitioning *without* ever needing to shard.

2. Compare range-based and hash-based sharding on two axes: how evenly
   they distribute load, and how cheaply they support a range query (e.g.
   "give me everything created this month"). Why does neither one win on
   both axes at once?

3. Walk through the math in this lesson: with 4 shards and naive modulo
   hashing, why does adding a 5th shard move roughly 80% of all keys,
   rather than the ~20% you'd hope for from adding 25% more capacity?
   What specifically about `hash(key) % N` changing to `hash(key) % (N+1)`
   causes this?

4. Explain, in your own words, why consistent hashing only moves roughly
   1/(N+1) of keys when going from N shards to N+1, instead of nearly all
   of them. What does "each shard owns an arc of the ring" have to do with
   limiting the blast radius of adding one shard?

5. `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
   used consistent hashing for load-balancer routing to per-instance
   caches. This lesson uses the same technique for database sharding.
   What's structurally identical between the two use cases, and what's
   different about the *cost* of getting the key-to-node mapping wrong in
   each one?

6. This module's earlier lessons (and the capstone) never actually shard
   Postgres — TaskForge and the anime catalog both run on a single
   instance. Make the case for why that's the right call for both systems
   as built, and describe what specific, measurable signal would tell you
   it's time to reconsider.
