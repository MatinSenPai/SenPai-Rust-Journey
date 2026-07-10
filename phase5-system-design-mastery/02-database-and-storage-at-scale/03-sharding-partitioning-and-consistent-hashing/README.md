# 02.3 — Sharding, partitioning, and consistent hashing

No code in this lesson. This is the deeper follow-up to a technique
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already introduced briefly for load-balancer routing — same underlying
math, applied here to splitting a *database* rather than routing requests
to a *cache*.

## Partitioning and sharding are not the same thing

These two words get used interchangeably constantly, including in casual
conversation among engineers who should know better, but they name
genuinely different things:

- **Partitioning** splits *one logical table* into multiple physical
  pieces that still live inside the *same database instance*. Postgres
  itself supports this natively (`PARTITION BY RANGE`/`LIST`/`HASH`) — a
  `jobs` table partitioned by `created_at` month, say, so old partitions can
  be dropped or archived cheaply and queries that filter by date only scan
  the relevant partition. The whole table is still one database, one
  connection pool, one machine (or one primary + its replicas) — partitioning
  is an *internal* storage optimization, invisible to most application code.
- **Sharding** splits data *across separate database instances* — different
  machines, different connection strings, no single query can `JOIN`
  across shards without the application (or a routing layer) stitching
  results together itself. Sharding is what you reach for when a single
  database instance's capacity (disk, write throughput, connection count)
  is the actual bottleneck, not just "this one table is unwieldy."

The relationship: sharding usually implies some partitioning scheme is
being used to decide *which* shard a row belongs on, but you can partition
without ever sharding (Postgres table partitioning on one instance), and in
principle you could shard without Postgres-style partitioning within each
shard (each shard just holds a whole unpartitioned table for its slice of
the keyspace). They solve different problems: partitioning makes one big
table more manageable; sharding makes one database's total capacity not be
your ceiling.

## Common sharding strategies

**Range-based sharding** — assign each shard a contiguous range of the
shard key (e.g. user IDs 1-1,000,000 on shard A, 1,000,001-2,000,000 on
shard B). Simple to reason about, and range queries within one shard's
range stay fast — but it's prone to **hot shards**: if user IDs are
assigned sequentially and your newest, most-active users all landed on the
highest range, one shard absorbs disproportionately more traffic than the
others while older shards sit comparatively idle.

**Hash-based sharding** — compute `hash(shard_key) % number_of_shards` and
route to that shard index. Spreads load much more evenly than naive
ranges (a good hash function scatters sequential IDs uniformly), which
avoids the "newest users all on one shard" problem — but range queries
("give me all users created this month") now have to fan out to *every*
shard, since consecutive keys are scattered across all of them by design.

**Directory-based sharding** — keep an explicit lookup table (`shard_key ->
shard_id`) in a separate service or table, and consult it on every routing
decision. Most flexible (you can move any individual key to any shard,
rebalance unevenly-loaded shards by hand, migrate one hot key without
touching the rest) — but that lookup table/service is now itself a
critical dependency and potential bottleneck, and every read/write pays an
extra lookup unless it's cached.

All three share one structural pain point: **resharding**. Adding a new
shard changes the total shard count, and for range-based or naive
hash-based sharding, changing the shard count changes *where nearly every
existing key belongs* — meaning nearly every row needs to be physically
moved to a new shard just to add one more machine. That's the specific
problem consistent hashing exists to solve.

## The resharding math, worked simply

Say you have 4 shards and route with naive modulo hashing:
`shard = hash(key) % 4`. A key whose hash is `17` lands on shard
`17 % 4 = 1`. Now add a 5th shard: the same key now routes to
`17 % 5 = 2` — a different shard. This isn't a coincidence for this one
key; changing the divisor in a modulo operation changes the result for
*almost every* input, because `%N` and `%(N+1)` agree only when a key's
hash happens to be a multiple of `N*(N+1)` (or close to it) — a small
fraction of the keyspace. Concretely: going from 4 shards to 5 shards with
plain modulo hashing, roughly **80% of all keys** compute a different
shard index than before (in general, going from N to N+1 shards moves
roughly `N/(N+1)` of all keys — 4/5 = 80% here). Every one of those keys'
rows has to be physically copied to a new shard before the cluster is
consistent again — for a large table, that's a massive, slow,
write-blocking (or at least write-complicating) migration just to add 25%
more capacity.

**Consistent hashing** fixes this by hashing shards *and* keys onto the
same conceptual ring (imagine hash values 0 to 2^32-1 arranged in a
circle). Each shard owns the arc of the ring between itself and the next
shard clockwise; a key is routed to whichever shard is the next one
clockwise from the key's own hash position. Adding a 5th shard means
placing one new point on that ring — it only takes over the arc between
itself and its counter-clockwise neighbor, stealing keys from *exactly
one* existing shard, not redistributing everyone's keys. With 4 shards
evenly spaced on the ring, each shard owns roughly 1/4 of the keyspace;
adding a 5th shard (also placed to divide the ring evenly, in the simple
case) means each of the 5 shards now owns roughly 1/5 — the new shard
takes about 1/5 of the keyspace, almost entirely from the shard(s)
adjacent to where it was inserted, and the other shards' key ownership is
*untouched*. So instead of ~80% of keys moving, only about **1/5 (20%) of
keys move** — exactly the amount that had to move for the new shard to
carry its fair share, and no more. (Real implementations place many
virtual points per physical shard around the ring, not one, specifically
to avoid any single shard getting an unlucky, unevenly large arc — but the
core "adding one shard only disturbs its immediate neighbors" property
holds either way.)

## Where you've already seen this technique

`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
introduced consistent hashing as a load-balancing strategy: routing a
request to a specific service instance based on a hash of the request's
cache key, so that instance's in-process cache (like the anime/manga
aggregator side-quest's `TtlCache`) stays warm instead of thrashing between
instances. That's the *identical* underlying technique — a hash ring
deciding "which node owns this key" — just applied to a different kind of
node: there, the nodes were stateless service replicas fronting a *shared*
cache concern; here, the nodes are the database shards themselves, and
which shard "owns" a key is a much higher-stakes decision, because moving
a key later means physically migrating its row, not just losing a cache
hit for one request. The math is exactly the same ring; what changes is
the cost of getting the mapping wrong.

No lesson in this repo has actually built a sharded database — every
Postgres-backed lesson here (the anime catalog, TaskForge) runs against a
single instance, which is itself a real, deliberate choice worth noticing:
none of these systems' data volumes or write throughput come close to
needing sharding, and sharding is a meaningfully more complex system to
operate (cross-shard queries, cross-shard transactions, rebalancing) than
a single well-indexed Postgres instance with read replicas. Reach for it
when a single instance's *write* capacity, specifically, is the proven
bottleneck — not preemptively.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
