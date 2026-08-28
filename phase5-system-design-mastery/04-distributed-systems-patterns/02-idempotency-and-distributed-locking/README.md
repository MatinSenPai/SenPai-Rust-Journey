# 04.2 — Idempotency and distributed locking

No code in this lesson.
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already introduced both ideas: the honest gap in `taskforge-api`'s
`POST /jobs` (no idempotency key, so a dropped-connection retry can
double-enqueue a job), and `taskforge-storage`'s `claim_next` using
Postgres's `SELECT ... FOR UPDATE SKIP LOCKED` as a distributed lock. Go
reread that lesson if it's been a while. This one doesn't re-explain
either concept — it goes one level deeper into how you'd actually *build*
each one for real.

## Designing an idempotency key system

The earlier lesson named the fix in one sentence: "the client generates a
unique idempotency key... the server checks 'have I already processed
this key?' before doing any work." Here's what that actually looks like
as a concrete design for `taskforge-api` — still a sketch, not code, but
specific enough that you could implement it directly.

**A new table**, alongside `jobs` in `taskforge-storage`'s migrations,
something like:

```sql
CREATE TABLE idempotency_keys (
    key            TEXT PRIMARY KEY,
    response_body  JSONB NOT NULL,
    status_code    INTEGER NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL
);
```

The `PRIMARY KEY` on `key` does the actual correctness work here — it's a
uniqueness constraint enforced by Postgres itself, not by application
logic that could race. That matters because idempotency is *exactly* the
kind of guarantee that's worthless if it isn't atomic: if "check for an
existing key" and "insert a new key" are two separate steps in application
code, two concurrent retries of the same request can both pass the check
before either has inserted anything, and you're back to double-processing.

**The middleware/handler flow**, for something like `POST /jobs` when the
client sends an `Idempotency-Key` header:

1. Client generates a UUID once, up front, and sends it as
   `Idempotency-Key: <uuid>` on the request (and on every retry of that
   *same* logical request — the client's responsibility is to reuse the
   key on retries, not generate a fresh one each time, or the whole scheme
   is pointless).
2. The handler attempts an `INSERT INTO idempotency_keys (key, ...)` — or
   checks-then-locks the row — *before* doing the real work (enqueueing
   the job). Whether this is "insert a placeholder row first, do the work,
   then update it with the real response" or "do the work inside the same
   transaction as the insert" is a real design decision: the placeholder
   approach handles the case where the handler crashes mid-request (the
   next retry sees an in-progress placeholder, not silence), at the cost
   of needing a way to represent "still processing" vs "done" in the row.
3. If the insert succeeds (key was new): do the real work (call
   `JobStore::enqueue`), then store the resulting response body and status
   code in that row, and return it normally.
4. If the insert fails because the key already exists (a `UNIQUE`
   violation): skip the real work entirely, fetch the stored
   `response_body`/`status_code` from the existing row, and return
   *that* — byte-for-byte the same response the client would have gotten
   the first time, including if the first attempt is what actually
   enqueued the job.

The key design property: the uniqueness check and the "claim" of that key
happen as one atomic database operation, the same way `claim_next`'s
`FOR UPDATE SKIP LOCKED` makes "check if a job is claimable" and "claim
it" atomic instead of two racy steps. Idempotency keys and distributed
locks are solving structurally similar problems — "make sure only one
winner exists for this piece of work" — which is part of why they show up
together in the same lesson.

## Distributed locking beyond a single Postgres row

The earlier lesson's example — `FOR UPDATE SKIP LOCKED` — is a distributed
lock, but it's specifically a *database-backed* one: the lock's
availability is tied to that one Postgres instance being reachable. That's
a fine tradeoff when you already have Postgres as your source of truth (as
TaskForge does) and don't want to introduce a second system just to
coordinate locks. But it's not the only approach, and it's worth knowing
the other two points on the spectrum.

**Redis-based locks.** The simplest version: `SET lock:job:123 <owner-id>
NX PX 30000` — `NX` means "only set if the key doesn't already exist" (an
atomic test-and-set), `PX 30000` attaches a 30-second TTL so the lock
auto-expires if the holder crashes without releasing it. Fast (an
in-memory `SET` is far cheaper than a Postgres transaction), and the TTL
solves the "holder died, lock never gets released" problem that a naive
lock without an expiry would have.

But naive TTL-based Redis locks have real correctness edge cases, and it's
worth being honest about them rather than treating Redis locks as a
drop-in, trivially-correct replacement for a database lock:

- **Clock drift and slow clients.** If the process holding the lock stalls
  (a long GC pause, a slow disk write, being descheduled by the OS) for
  longer than the TTL, Redis expires the lock while the original holder
  still thinks it owns it — and hands the lock to a second client. Now
  *two* processes believe they hold the same lock simultaneously, which is
  the exact failure the lock existed to prevent. This isn't a rare
  theoretical edge case; slow-client pauses that exceed a lock TTL are a
  well-documented real-world failure mode.
  - **Redlock** is an algorithm (proposed by Redis's author) meant to
    harden this: acquire the same lock against a majority of N independent
    Redis instances, with a tight timing budget, so a single instance's
    clock or availability issue can't unilaterally grant a bad lock.
    It's a genuine improvement over a single-instance `SETNX`, but it's
    also been the subject of real, substantive correctness critiques
    (most famously from Martin Kleppmann) — the core objection being that
    it still fundamentally depends on synchronized clocks and bounded
    process pauses, assumptions that don't hold under all real-world
    failure conditions. Worth knowing Redlock exists and what problem it's
    reaching for, but not worth treating as a solved problem the way, say,
    Postgres's `FOR UPDATE` is.

**Zookeeper/etcd-based locks.** Both are consensus-backed coordination
services (Zookeeper uses ZAB, etcd uses Raft) purpose-built for exactly
this kind of problem — not just locking, but leader election, service
discovery, and distributed configuration generally. A lock here is backed
by a quorum of nodes agreeing, not a single instance's local state or a
best-effort TTL, which gives it the strongest correctness guarantees of
the three approaches: no single node's clock drift or crash can silently
hand the same lock to two clients, because a majority has to agree on
every state change. The cost is operational: running and correctly
operating a Zookeeper or etcd cluster (itself a distributed system that
needs monitoring, upgrades, and quorum maintenance) is meaningfully more
overhead than "have Postgres" or "have Redis," both of which you likely
already run for other reasons.

## Comparing the three

| Approach | Correctness | Speed | Operational cost |
|---|---|---|---|
| Postgres row lock (`FOR UPDATE SKIP LOCKED`) | Strong, *if* you already trust Postgres as your source of truth | Good enough for most workloads; bounded by transaction/query latency | None extra — you already run Postgres |
| Redis (`SETNX` + TTL, or Redlock) | Weaker; naive TTL locks have real correctness gaps under clock drift/slow clients | Fastest — in-memory | Low if you already run Redis; Redlock adds real operational and reasoning complexity |
| Zookeeper / etcd | Strongest — consensus-backed | Slower than Redis, comparable to or slower than Postgres | Highest — a whole extra distributed system to run correctly |

`taskforge-storage` chose the first option not because it's universally
"best," but because Postgres was already the system of record — reaching
for Redis or Zookeeper to lock rows in a table that lives in Postgres
would mean trusting *two* systems to agree about the same piece of state,
which is strictly more that can go wrong, not less. The general rule: use
the coordination mechanism of whichever system already holds the data
being locked, unless a *specific*, measured bottleneck (lock contention
your database can't sustain at your actual scale) forces you elsewhere.

## Next

No `cargo test` for this lesson — it is a reading lesson.
