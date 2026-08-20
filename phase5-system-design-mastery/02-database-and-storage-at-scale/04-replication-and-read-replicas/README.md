# 02.4 — Replication and read replicas

No code in this lesson. Every Postgres instance you've stood up in this
repo (local dev, CI, the `docker-compose.yml` in `capstone-taskforge/`) has
been a single node — this lesson is about what changes, and what new
problems show up, the moment you add more.

## Leader-follower replication

The standard setup: one **primary** (also called a leader) accepts all
writes; one or more **replicas** (followers) continuously receive a stream
of the primary's changes (Postgres does this via **streaming replication**
of its write-ahead log, the same WAL that durability, from lesson 02 of
this module, relies on) and apply them locally, keeping their own copy of
the data up to date. Reads can be served from either the primary or any
replica; writes can only go to the primary — a replica that accepted
writes directly would need some way to reconcile conflicting writes with
the primary, which is a fundamentally harder problem (multi-primary
replication exists, but it's a different, more complex architecture with
its own conflict-resolution machinery).

## Synchronous vs. asynchronous replication

This is a real consistency-vs-latency tradeoff, not a free choice:

- **Synchronous replication** — the primary waits for at least one replica
  to confirm it has received (and, depending on configuration, applied)
  a write *before* telling the client the write succeeded. This guarantees
  a replica is never more than zero committed transactions behind the
  primary for anything the client has been told succeeded — strong
  consistency — at the cost of every write's latency now including a
  round trip to the replica, and (worse) the primary being unable to
  confirm *any* write if that replica is unreachable, which is itself an
  availability cost.
- **Asynchronous replication** — the primary commits and tells the client
  "done" immediately, and ships the change to replicas in the background,
  whenever it gets to it. Writes are fast and don't depend on replica
  health at all — but a replica can lag behind the primary by anywhere
  from milliseconds to (under load or network trouble) seconds or more,
  and if the primary crashes before a change ships, that committed write
  can be lost from the replica's perspective (it never saw it) even though
  the client was already told it succeeded.

Postgres defaults to asynchronous replication precisely because most
systems value write latency and primary availability over the stronger
guarantee — the same kind of tradeoff CAP theorem forces during a
partition (`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`),
just applied continuously rather than only during a network split.

## Read replicas: scaling reads, not writes

The most common reason to add replicas isn't disaster recovery (though
they help with that too) — it's **read scaling**. If a system is
read-heavy (most web apps are: many more `GET`s than `POST`s/`PATCH`es),
routing reads across several replicas while writes stay concentrated on
the primary multiplies your read capacity without touching write capacity
at all. This is a genuinely different scaling lever than sharding (lesson
03): sharding splits *both* reads and writes across machines by splitting
the *data*; read replicas keep the *full dataset* on every node and split
only the *read traffic*, which is much simpler to reason about (every
replica has everything) but does nothing for write throughput, since every
write still funnels through one primary.

## Replication lag as a real, concrete problem

Because asynchronous replication doesn't guarantee replicas are current,
"how stale can a replica be" isn't a hypothetical edge case — it's a
routine, daily occurrence proportional to how much write load the primary
is under. The classic symptom:

**Worked example.** A user posts a comment. The `POST` hits the primary,
commits, returns `201 Created`. The client's UI immediately re-fetches the
comment list — a `GET` — and that `GET` happens to get load-balanced to a
read replica that hasn't yet received the just-committed `INSERT` (maybe
it's 200ms behind, an entirely normal amount of lag under moderate load).
The user's own comment is missing from the list they just refreshed. This
is the **read-your-writes consistency** problem: a system can be
eventually consistent (the replica *will* catch up) while still violating
the much more specific, user-facing expectation that "I should always be
able to see the effect of my own action immediately," even if I'm fine not
seeing *other* users' very-recent actions right away.

Two standard fixes:

1. **Read-from-primary-after-write** — after a client performs a write, route
   that same client's *next* read (or reads, for some short window) to the
   primary instead of a replica, specifically to guarantee they see their
   own change. Simple and correct, but if overused it defeats the purpose
   of having read replicas at all — the trick is scoping it narrowly (e.g.
   only the specific resource just written, only for a few seconds) rather
   than routing a user's entire session to the primary.
2. **Session-sticky routing** — once a client has performed a write, pin
   that client's session to the same replica (or the primary) for some
   window of time, so subsequent reads at least see a *monotonically
   advancing* view (never see the replica "go backwards" relative to what
   they've already observed), even if that replica is a little behind the
   absolute latest primary state. Weaker than read-from-primary (doesn't
   guarantee the very latest write is visible), but spreads load more
   evenly than routing every post-write read to the primary.

Neither fix is "solve replication lag" — replication lag itself is
inherent to asynchronous replication and isn't going away. Both fixes are
about narrowing *which* reads need the stronger guarantee, rather than
paying the synchronous-replication cost for every read in the system.

## What would have to change in TaskForge to add read replicas

`taskforge-api`'s handlers (`capstone-taskforge/taskforge-api/src/handlers.rs`)
all go through a single `Arc<dyn JobStore>` on `AppState` — `get_job`,
`list_jobs`, `enqueue_job`, `cancel_job` all call the same trait object for
both reads and writes, and today that trait object is backed by exactly
one Postgres connection pool pointed at exactly one instance (see
`taskforge-storage/src/postgres.rs`'s `PostgresJobStore::connect`). If
TaskForge needed to scale reads, two shapes of change would work, both
compatible with the ports-and-adapters boundary `docs/adr/0001-architecture-overview.md`
already established:

- **Split `JobStore` into a write path and a read path** — e.g. a
  `JobReader` trait with `get`/`list` and a `JobWriter` trait with
  `enqueue`/`claim_next`/`mark_succeeded`/`mark_failed`/`cancel`, with
  `AppState` holding one `Arc<dyn JobWriter>` (pointed at the primary) and
  one `Arc<dyn JobReader>` (pointed at a replica, or a pool that spreads
  reads across several). `get_job`/`list_jobs` would switch to the reader;
  everything else keeps using the writer. This is the most explicit
  option: it's visible in the type signatures which operations are
  read-vs-write-sensitive, and it would force a conscious decision at each
  call site about which one to use — including, importantly, deciding that
  `enqueue_job`'s immediate-response path (a client that just enqueued a
  job might reasonably expect `get_job` right after to see it) needs
  read-your-writes handling, exactly the problem above.
- **Keep one `JobStore` trait, hide the routing inside a single
  implementation** — a `ReplicatedPostgresJobStore` that holds both a
  primary pool and one or more replica pools internally, and routes each
  trait method to the right one itself (writes to primary, `get`/`list`
  to a replica, with some read-your-writes logic — e.g. routing `get_job`
  to the primary for a short window after that specific job was last
  written). This keeps every call site in `taskforge-api` unchanged, at
  the cost of the routing logic being less visible — a future contributor
  reading a handler's `state.store.get(...)` call can't tell from the type
  alone whether it's hitting the primary or a replica.

Either way, the trait-boundary architecture from ADR-0001 is exactly what
makes this an *additive* change rather than a rewrite — the same property
ADR-0002 already leaned on when it said a future `KafkaJobStore` could be
introduced "behind the same `JobStore` trait." Read replicas are just
another case of "the interface the rest of the system depends on doesn't
need to change for the implementation behind it to get smarter."

## Next

No `cargo test` for this lesson — it is a reading lesson.
