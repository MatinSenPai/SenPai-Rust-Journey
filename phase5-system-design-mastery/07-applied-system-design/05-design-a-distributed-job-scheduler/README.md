# 07.5 — Design a distributed job scheduler

No new code in this lesson — you already built this. `capstone-taskforge`
**is** the answer to "design a distributed job scheduler," built crate by
crate across this entire curriculum. This lesson is a deliberate second
pass: read your own system again with every Phase 5 module's vocabulary
in hand, and see how much of it TaskForge already had a real answer for,
and where it honestly doesn't.

## Re-reading TaskForge as a system design interview answer

A system design interview for "design a distributed job scheduler"
typically wants you to cover, roughly in this order: requirements,
data model, the claiming/locking mechanism, failure handling, and
scaling. Walk through what TaskForge already has for each:

- **Data model** — `taskforge-core`'s `Job`/`JobStatus` state machine
  (`Pending → Running → {Succeeded, Failed→Retrying, DeadLetter}`). Go
  reread `docs/adr/0001-architecture-overview.md` if it's been a while —
  this is the ADR-first discipline the capstone was built with, and it's
  exactly the "design before code" structure a live system design
  interview expects out loud.
- **Claiming/locking** — `taskforge-storage`'s `claim_next`, `SELECT ...
  FOR UPDATE SKIP LOCKED`. This *is* Module 4's distributed-locking
  lesson, already implemented, not a hypothetical. If Module 4's
  `02-idempotency-and-distributed-locking` lesson's Postgres-lock-vs-Redis-lock-vs-Zookeeper
  comparison felt abstract when you read it, it shouldn't now: TaskForge
  made the Postgres choice, for the reasons `docs/adr/0002-postgres-backed-queue.md`
  already wrote down.
- **Failure handling** — `taskforge-worker`'s per-job panic isolation and
  exponential backoff + jitter (Module 4's `03-resiliency-patterns`
  lesson, already implemented — go find the actual `compute_backoff`
  function and its call sites if you haven't looked at that code in a
  while).
- **Unique IDs** — `taskforge-core`'s `JobId(Uuid)`, a real instance of
  Module 4's `04-unique-id-generation` lesson's "coordinated writers need
  either a shared sequence or collision-resistant randomness" tradeoff,
  decided in favor of UUIDs specifically because jobs get created by
  potentially many `taskforge-api` replicas concurrently.
- **Scaling** — `taskforge-worker` scales horizontally today (run more
  worker processes, each independently claiming from the same Postgres
  queue, zero coordination code needed between them — Module 4's
  `05-scalability-strategies` lesson's functional-partitioning and
  stateless-worker discussion, already true of your own code).

## Where TaskForge honestly doesn't have an answer yet

This repo's own agents and its author found these gaps while building
and verifying TaskForge — they're documented, not hidden:

- **No automatic reclaim-on-crash.** If a worker claims a job (marks it
  `Running`) and then crashes before finishing, nothing currently notices
  and re-queues that job — `docs/adr/0004-worker-failure-handling.md`
  says so explicitly. A real fix needs a "claimed but not heard from in N
  seconds" sweep, re-queuing anything past that threshold — this is
  genuinely unbuilt, a real next task, not a solved problem you missed.
- **No idempotency key on `POST /jobs`.** A client retry after a dropped
  connection can double-enqueue a job — Module 4's
  `02-idempotency-and-distributed-locking` lesson's `idempotency_keys`
  table sketch would be the fix, applied to `taskforge-api` specifically.
- **Single Postgres, CP-leaning, no read replicas.** Module 2's
  replication lesson's read-your-writes discussion applies directly:
  `taskforge-api`'s handlers all read and write through the same
  `Arc<dyn JobStore>` today — if `GET /jobs` traffic ever needed to scale
  independently of write traffic, that trait would need the
  reader/writer split that lesson sketched.

## Your task

There's no `todo!()` to fill in — the exercise is a **written retrospective**.
Answer `CHECKPOINT.md` seriously, in your own words, referencing the actual
files named above (open them, don't work from memory). If you want to go
further: pick one of the three gaps above and actually implement it against
the real `capstone-taskforge` crates — the ADRs already model the
process (`docs/adr/000N-<decision>.md` written *before* the code) to follow
if you do.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
