# 02.1 — SQL vs NoSQL, and choosing a database

No code in this lesson. Every database you've touched so far in this repo
has been Postgres — the anime catalog, the blog `posts`/`comments` schema,
TaskForge's job queue, all of it. That's not an accident and it's not
because "SQL is the default until you outgrow it." This lesson is about
*why* it kept being the right call, and about what "NoSQL" actually means
(spoiler: it means at least four genuinely different things wearing one
marketing label).

## "NoSQL" is not one thing

If you've only ever heard NoSQL discussed as "the alternative to SQL," it's
easy to picture it as a single technology with a single set of tradeoffs.
It isn't. "NoSQL" is an umbrella term for at least four distinct data
models, each solving a different shape of problem:

- **Key-value stores** (Redis, DynamoDB, etcd) — the simplest possible
  model: `get(key) -> value`, `set(key, value)`. No query language, no
  schema, often no relationships at all. Extremely fast, extremely simple,
  and that's the whole feature set. Good for caches, session storage,
  feature flags — anything you look up by exact key and never need to query
  by any other field.
- **Document stores** (MongoDB, Couchbase) — each record is a
  self-contained, semi-structured document (usually JSON-shaped), and
  different documents in the same collection can have different fields.
  Good for data that's naturally nested and doesn't need much cross-record
  referential integrity — a product catalog where every product type has
  wildly different attributes, a content-management system's page bodies.
- **Wide-column stores** (Cassandra, HBase, Bigtable) — rows can have
  millions of columns, each row can have a different set of columns, and
  data is physically partitioned by a row key across many machines from
  day one. Built for write-heavy, horizontally-distributed workloads at a
  scale where "one machine holds the whole table" stopped being an option —
  time-series data, sensor telemetry, activity feeds.
- **Graph databases** (Neo4j, Amazon Neptune) — nodes and edges are
  first-class, and traversing relationships ("friends of friends of
  friends") is the thing the engine is optimized for, in a way a relational
  `JOIN` chain gets expensive at doing past 2-3 hops. Good for social
  graphs, recommendation engines, fraud-detection ring-finding.

These four have almost nothing in common with each other beyond "not
relational tables with a fixed schema and `JOIN`." Lumping them together as
"NoSQL" is a bit like lumping "bicycle," "cargo ship," and "helicopter"
together as "not a car" — technically accurate, useless for choosing one.
The question is never "SQL or NoSQL," it's "which of these five specific
models (relational included) fits this specific access pattern."

## The real tradeoffs

Three axes actually matter, and none of them are "which one is faster" in
the abstract (that question is close to meaningless without a specific
workload):

**Schema flexibility vs. integrity guarantees.** A relational schema
(`CREATE TABLE anime (id BIGSERIAL, title TEXT NOT NULL, status TEXT NOT
NULL, rating SMALLINT)`) forces every row to have the same shape, and a
`NOT NULL`/`CHECK`/foreign-key constraint is enforced by the database
itself, on every write, forever — you cannot accidentally insert an anime
with no title, because Postgres refuses the write. A document store lets
you add a field to new documents without a migration and without touching
old ones — genuinely useful when your data's shape is still evolving fast
or is inherently heterogeneous — but "is every document valid" becomes
something your application code has to enforce (or not), not something the
database guarantees for you.

**Join support.** Relational databases are built around the idea that data
normalizes into separate tables and gets recombined at query time — `posts`
`JOIN` `comments` `ON comments.post_id = posts.id`, exactly what
`phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`
does. Most NoSQL stores either don't support joins at all or actively
discourage them — the expectation is that you either denormalize (embed a
post's comments *inside* the post document) or do the join in application
code (which is exactly the N+1 trap that lesson teaches you to avoid). Not
having joins isn't automatically worse — it's a different bet: pay the cost
once at write time (keep denormalized copies in sync) instead of at read
time (join across tables every query).

**Horizontal scaling ease.** This is the one NoSQL genuinely tends to win
on on average, though it's not universal: Cassandra and DynamoDB were
designed from day one to partition data across many machines with no
single node ever holding the whole dataset, and to keep accepting writes
even if some nodes are unreachable (the AP-leaning choice from
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`'s
CAP discussion). A single Postgres primary, by contrast, has one node doing
all writes — you can scale reads with replicas (lesson 04 in this module)
and scale writes with sharding (lesson 03), but both of those are things
you *add*, deliberately, when you need them — they're not the default
shape the database starts in.

## A practical decision framework

Not "SQL is legacy, NoSQL is web-scale" — that's a slogan, not an
engineering argument, and it was wrong even at the height of its
popularity. A better set of questions, roughly in the order you should ask
them:

1. **Does correctness depend on relationships between records staying
   consistent?** (A job's status must never say "succeeded" while a
   `retrying` row for the same job also exists; an order total must equal
   the sum of its line items.) If yes, you want strong integrity
   guarantees and probably joins — lean relational.
2. **Is the access pattern "look up by exactly one key, fast, at huge
   scale"?** (A session token, a cache entry, a feature flag.) If the data
   has no meaningful relationships to anything else, a key-value store is
   simpler and faster than forcing it into a table.
3. **Is the data's shape genuinely heterogeneous or fast-evolving, and are
   cross-record integrity constraints not that important?** A document
   store trades schema rigidity for flexibility — reasonable when the
   rigidity wasn't buying you much anyway.
4. **Does write volume alone, sustained, exceed what a well-tuned single
   Postgres primary plus read replicas can handle, even after indexing and
   query optimization?** This is a much higher bar in practice than most
   people assume (a single modern Postgres instance handles tens of
   thousands of writes/second on decent hardware) — but if you're
   genuinely past it, or need multi-region active-active writes, a
   horizontally-native store (Cassandra, DynamoDB) starts to make sense.
5. **Do you need graph traversal as the primary query shape?** If "find
   everyone within 3 hops of this person" is a core, frequent query, a
   graph database's traversal performance is a real, not cosmetic,
   advantage over recursive SQL `JOIN`s.

Most backend systems — including every one you've built in this repo —
answer "yes" to question 1 and "no" to the rest, which is exactly why
Postgres shows up everywhere here.

## Why TaskForge is Postgres, concretely

`capstone-taskforge/docs/adr/0002-postgres-backed-queue.md` already
explains the *operational* reasoning (one fewer system to run, "you
probably already have Postgres"). The *data-model* reasoning is just as
important and is worth stating on its own: `capstone-taskforge/taskforge-core/src/job.rs`'s
`JobStatus` enum encodes a state machine —

```rust
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Retrying { attempt: u32, next_attempt_at: DateTime<Utc> },
    Failed { error: String },
    DeadLetter { error: String },
}
```

— and `docs/adr/0003-job-state-machine.md` is explicit that a job must
*never* be observable in a nonsensical combination of states (e.g.
"succeeded, but also has a pending retry"). That's exactly the kind of
invariant a strongly-consistent, single-writer relational database is good
at protecting: `taskforge-storage/src/postgres.rs`'s `claim_next` uses
`FOR UPDATE SKIP LOCKED` inside one atomic statement precisely so that
"exactly one worker transitions this job from `Pending`/`Retrying` to
`Running`" is a guarantee the database itself enforces, not something
application code has to hope about. A wide-column or document store that
prioritizes availability and horizontal write scaling over strict
consistency (Cassandra-style "eventually consistent" writes, for instance)
would make that exact guarantee much harder to get — you'd risk two
replicas briefly disagreeing about whether a job is claimed, which for a
"like" counter is a shrug and for "did we just run this job twice" is a
real bug. TaskForge's job-state tracking needs strong consistency on status
transitions far more than it needs horizontal write scaling — ADR-0002's
own numbers back this up: "thousands of jobs/second on modest hardware" is
enough for the vast majority of real systems, and the trait boundary
(`JobStore`, from `docs/adr/0001-architecture-overview.md`) means a
horizontally-scaled backend could be swapped in later, additively, if that
ever changed.

The same reasoning applies to `phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`:
an anime's `status`/`rating` fields have real integrity constraints
(`validate_rating` rejects a rating outside 1-10 at the database boundary,
`WatchStatus::parse` refuses to silently accept a corrupt value), and
nobody needs that table sharded across ten machines — it's a textbook
"question 1 says yes, nothing else says yes" case.

## Next

No `cargo test` for this lesson — it is a reading lesson.
