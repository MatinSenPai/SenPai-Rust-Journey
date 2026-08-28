# 04.5 — Scalability strategies

No code in this lesson.
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already covered vertical vs. horizontal scaling and stateless service
design, using `taskforge-api` as the worked example (no per-request
mutable state, so any replica can serve any request). Go reread that
section — it isn't repeated here. This lesson covers the rest of the
menu: splitting *what* runs where (functional partitioning), scaling
*automatically* rather than by hand (auto-scaling), and the fact that
"scaling" isn't one problem — reads and writes scale by genuinely
different mechanisms.

## Functional partitioning: splitting by feature, not just running more copies

Horizontal scaling (more copies of the same service) assumes the service
is small and uniform enough that "run more of it" is the right lever.
**Functional partitioning** (also called functional decomposition) is the
other axis: instead of N identical copies of one monolith, split the
system by feature/domain into separate services that can each be deployed,
scaled, and failed independently.

`capstone-taskforge` is itself already partitioned this way, deliberately,
across its seven crates — not as one monolithic binary that does
everything, but as separable roles: `taskforge-api` (HTTP surface),
`taskforge-worker` (job execution), `taskforge-scheduler` (presumably
recurring/delayed job triggering), `taskforge-admin-bot`, `taskforge-cli`,
with `taskforge-core` and `taskforge-storage` as shared libraries
underneath. The payoff of this split specifically for scaling: `taskforge-api`
and `taskforge-worker` have completely different load profiles —
API traffic scales with client request volume, worker load scales with
job *volume* and how expensive each job type's handler is to run — and
functional partitioning means you can scale each independently. If job
volume spikes but API traffic doesn't, you add worker replicas without
touching the API tier at all; a monolith that bundled both concerns into
one process would force you to scale the whole thing together even though
only half of it is actually under load.

## Stateless service design as the prerequisite (cross-reference only)

Both horizontal scaling *and* functional partitioning depend on the same
underlying property: each independently-scaled piece has to be safe to
run as multiple instances with no shared in-process state. The earlier
lesson already covered exactly why this holds for `taskforge-api` (every
handler re-derives state from `Arc<dyn JobStore>`, never an in-process
cache) — that reasoning applies unchanged to `taskforge-worker`, which is
the concrete example for the next section.

## Auto-scaling: reactive scaling based on real signals

Everything above describes *how* to split load across more instances;
auto-scaling decides *when* to add or remove them, automatically, based on
a live metric crossing a threshold rather than a human deciding by hand.
Common signals:

- **CPU utilization** — the oldest, simplest signal (e.g. "add an instance
  once average CPU across the fleet exceeds 70% for 5 minutes"). Easy to
  reason about, but a poor proxy for load in I/O-bound services — a
  service mostly waiting on database or network calls can be badly
  overloaded (long queues, high latency) while CPU sits low, because it's
  not compute-bound in the first place.
- **Queue depth** — directly relevant to `taskforge-worker`'s shape: scale
  worker replicas based on how many `Pending`/eligible-`Retrying` jobs are
  sitting in `taskforge-storage`'s `jobs` table waiting to be claimed. This
  is a much more honest signal for a queue-consumer service than CPU,
  because it measures the thing you actually care about — is work piling
  up faster than it's being drained — directly, rather than inferring it
  from a proxy metric.
- **Request rate** — for request-serving tiers like `taskforge-api`,
  scaling on incoming requests/second (or on a derived metric like
  in-flight request count, closer to what "least-connections" load
  balancing already measures per-instance) responds faster to a traffic
  spike than CPU does, since request volume can jump before CPU usage
  fully reflects it.

The general shape is always the same: pick a metric that's causally close
to "is this tier actually falling behind," set a threshold and a cooldown
(so you don't thrash — scale up, immediately scale back down, repeat), and
let the orchestration layer add/remove replicas automatically rather than
paging a human to do it by hand at 3am.

## Read scaling vs. write scaling: not the same problem

This is the distinction that trips people up most: "scaling the database"
sounds like one problem, but reads and writes hit genuinely different
ceilings and need genuinely different fixes.

**Reads scale easily** with read replicas: a primary handles writes, and
one or more replicas continuously stream the primary's changes and serve
read-only queries. Adding read capacity is close to free — spin up another
replica, point more read traffic at it. The cost is a consistency one, not
a scaling one: replicas lag the primary by some (usually small) amount,
so a replica's read can be stale relative to the primary — the exact
"eventual consistency" tradeoff from this module's first lesson, just
applied within a single database's own replication rather than across a
whole distributed store like DynamoDB.

**Writes don't scale the same way.** Every write still has to go through
the primary (or, in a multi-primary setup, has to be reconciled across
primaries, which reintroduces the CAP tradeoff directly) — adding more
read replicas does nothing for write throughput, because replicas don't
absorb writes, they only copy what the primary already committed. Once a
single primary's write throughput (or disk, or connection count) is the
actual bottleneck, the only way past it is **sharding/partitioning** —
splitting the dataset itself across multiple independent primaries, each
owning a slice of the keyspace, so writes for different keys can happen in
parallel on different machines. This repo's Module 2 covers that in full:
`phase5-system-design-mastery/02-database-and-storage-at-scale/03-sharding-partitioning-and-consistent-hashing`
— worth reading if you haven't, since it's the direct continuation of
"read replicas aren't enough anymore."

`capstone-taskforge` currently sits comfortably on the "read replicas
would be enough, if needed at all" side of that line — a single Postgres
instance, per `taskforge-storage`'s `PostgresJobStore`, with no sharding
in scope, which is the right call at this system's actual scale (the
sharding lesson makes the same point: reach for sharding when write
throughput is a *proven* bottleneck, not preemptively).

## Where TaskForge already scales horizontally: the worker side

`taskforge-worker`'s `WorkerPool` has an in-process `concurrency` setting
(`with_concurrency`, default 4) — that's *concurrency within one process*,
running multiple async worker loops inside a single `taskforge-worker`
binary. Horizontal scaling is the layer above that: run **multiple
independent `taskforge-worker` processes**, each with its own `WorkerPool`,
each on possibly different machines, all pointed at the same Postgres
instance via `PostgresJobStore`. Every one of those processes' worker
loops calls the identical `store.claim_next()` — the `FOR UPDATE SKIP
LOCKED` query this module's second lesson covers in depth — and Postgres
itself is what guarantees two processes (or two loops within the same
process) never claim the same row. That's the entire mechanism: nothing
in `taskforge-worker`'s code needs to know how many other worker processes
exist, or coordinate with them directly at all, because the coordination
already lives in the one place all of them talk to — the same "let the
system of record do the coordinating" principle
`02-idempotency-and-distributed-locking` makes about locks in general.

Concretely, this means scaling `taskforge-worker` to handle a job-volume
spike is exactly "run more `taskforge-worker` processes" — no code
change, no reconfiguration of existing workers, nothing to rebalance by
hand. That property — adding capacity by running more of the same
stateless unit, safely, with zero coordination logic of its own — is
horizontal scaling working exactly as intended, and it's only possible
because `claim_next`'s locking already made "two workers, same job" a
solved problem before scaling was ever a consideration.

## Next

No `cargo test` for this lesson — it is a reading lesson.
