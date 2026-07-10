# 06.1 — CAP, scaling, load balancing, idempotency, distributed locking

No code in this lesson. Every idea below is something you've already built
somewhere earlier in this repo — the goal here is attaching the vocabulary
that shows up in system design interviews and incident postmortems to a
decision you already made, not learning anything conceptually new.

## CAP theorem

A distributed system that stores data across more than one node faces an
unavoidable tradeoff the instant a network partition happens (some nodes
can't reach others): you can keep serving requests everywhere
(**Availability**) or guarantee every node sees the same, fully up-to-date
data (**Consistency**), but during the partition itself, not both.
(**Partition tolerance** isn't really a choice — real networks partition
whether you plan for it or not — which is why CAP is usually phrased as "pick
C or A, you don't get to pick P.")

**Where you already made this choice:** `taskforge-storage`'s
`PostgresJobStore` is a single Postgres instance. A single-writer Postgres
setup is **CP**-leaning: if a replica can't reach the primary, it refuses to
serve a write rather than risk two nodes disagreeing about a job's status —
consistency wins, at the cost of availability during that partition. Compare
to something like Cassandra or DynamoDB, which are **AP**-leaning by
default: every node keeps accepting reads and writes during a partition, and
inconsistencies get reconciled later (sometimes the client sees stale data
in the meantime). Neither choice is "correct" in the abstract — TaskForge
chose CP because a job silently double-running (from two nodes disagreeing
about who owns it) is worse than the system being briefly unavailable during
a rare partition; a "like" counter on a social media post might reasonably
make the opposite tradeoff, since a few seconds of a stale count is harmless
and total unavailability isn't.

## Horizontal vs. vertical scaling

**Vertical scaling**: give one machine more CPU/RAM. Simple, but has a hard
ceiling (there's a biggest machine your cloud provider sells) and a single
point of failure (that one machine dies, everything's down). **Horizontal
scaling**: run more copies of the same service across more machines, behind
something that spreads traffic across them. No hard ceiling, and one
instance dying doesn't take the whole service down — but it only works if
any instance can handle any request, which requires the service to be
**stateless**.

**Where you already relied on this:** `taskforge-api` holds no per-request
mutable state of its own — every handler re-derives everything it needs
from `Arc<dyn JobStore>` (ultimately, Postgres). That's exactly what makes
it safe to run N copies of `taskforge-api` behind a load balancer: request
17 can land on instance A and request 18 on instance B, and neither
instance needs to know anything about the other, because neither instance
is the source of truth for anything — Postgres is. If `taskforge-api`
instead cached job state in an in-process `HashMap` the way this repo's
*teaching* examples do (deliberately, for infra-free lessons), running two
replicas would immediately produce two silently-diverging views of the
world — exactly the risk called out in the observability side-quest's
checkpoint about the anime aggregator's in-memory cache.

## Load balancing

A load balancer sits in front of multiple service instances and decides
which one handles each incoming request. Common strategies:

- **Round-robin** — requests 1, 2, 3, 4 go to instances A, B, A, B in strict
  rotation. Simple, works fine when every request costs roughly the same.
- **Least-connections** — send the next request to whichever instance
  currently has the fewest in-flight requests. Better than round-robin when
  request cost varies a lot (one slow request shouldn't get piled on with
  more work just because it's "its turn").
- **Consistent hashing** — route based on a hash of some request property
  (e.g. a cache key, or a user id), so the *same* key reliably lands on the
  *same* instance every time, rather than being spread arbitrarily.

Consistent hashing matters specifically for caching layers: if you ran
multiple instances each with their own **in-process** cache (like the
anime/manga aggregator side-quest's `TtlCache`) behind plain round-robin,
the same title id would bounce between instances on every request, and
each instance's cache would independently see mostly misses — you'd pay
the "expensive lookup" cost almost as often as with no cache at all.
Routing consistently by id means the same instance keeps handling the same
titles, so its cache actually gets warm. (The other, more common fix —
covered in the caching lesson itself — is to stop caching in-process at
all and cache in a shared store like Redis that every instance can reach;
consistent hashing at the load balancer is the alternative when you
specifically want to keep caching local to each instance for speed.)

## Idempotency

An operation is **idempotent** if doing it twice has the same effect as
doing it once. `GET` and `DELETE` are idempotent by HTTP convention — asking
twice, or deleting an already-deleted thing, shouldn't change the outcome.
`POST` (creating something new) is usually **not** idempotent by
default — and that's a real problem the instant a client can retry a
request it's not sure succeeded (a timeout, a dropped connection — the
client genuinely cannot tell "the server never got this" from "the server
got this, did the work, and the response got lost on the way back").

**A real gap this exposes:** `taskforge-api`'s `POST /jobs` handler, as
built in this repo, has no idempotency key. If a client's connection drops
right as the server finishes enqueueing a job but before the response makes
it back, a naive retry enqueues the job a *second* time — same job, run
twice. The standard fix: the client generates a unique idempotency key
(e.g. a UUID) up front and sends it with the request; the server checks
"have I already processed this key?" before doing any work, and if so,
returns the *original* response instead of enqueueing again. This is a
genuine, honest gap in TaskForge as currently built, not a solved problem —
a natural next addition if you wanted to keep extending the capstone.

## Distributed locking

A plain `Mutex` (or `RwLock`) only protects against concurrent access
*within one process* — it lives in that process's memory, and a second
process (a second replica of your service, on a different machine or even
just a different OS process on the same one) has no way to see or respect
it. Once your job-claiming logic needs to be safe across *multiple
independent worker processes*, you need a lock that lives somewhere every
process can see: the database itself.

**You already implemented distributed locking** without necessarily naming
it that: `phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue`'s

```sql
SELECT id FROM toy_jobs WHERE status = 'pending'
ORDER BY id FOR UPDATE SKIP LOCKED LIMIT 1
```

is exactly a distributed lock: `FOR UPDATE` locks the selected row so no
other transaction (running in any process, on any machine, connected to the
same Postgres) can also select it, and `SKIP LOCKED` means a second worker
racing to claim a job doesn't block waiting for the first worker's lock —
it just moves on to the next available row. `taskforge-storage`'s real
`claim_next` uses the identical pattern. The "lock" here isn't a language
primitive at all — it's a guarantee the database itself provides, which is
precisely why it works across processes and machines where a `Mutex` never
could.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
