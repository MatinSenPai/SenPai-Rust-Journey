# ADR-0001: Architecture overview

## Status
Accepted

## Context

TaskForge is the capstone of this curriculum: a production-shaped background
job/task-queue engine — the same category of tool as Sidekiq (Ruby), Celery
(Python), or BullMQ (Node). The goal isn't a toy that merely compiles; it's
a genuinely marketable artifact demonstrating that everything from Phase 0
through Phase 4 composes into a real system, plus real system-design
judgment about failure modes (crashed workers, duplicate delivery, poison
jobs) that a CRUD app never forces you to confront.

A hard environment constraint shapes this ADR directly: this repository (and
any learner's machine, and CI) cannot assume a live PostgreSQL instance is
always running. The architecture must let every crate's core logic be fully
unit-tested with **zero external infrastructure**, while still providing a
real, correct Postgres-backed implementation for actual use.

## Decision

**Ports and adapters (hexagonal architecture).** `taskforge-core` defines
the domain model (`Job`, `JobStatus`, `JobError`) and a `JobStore` trait —
the "port." It has zero I/O dependencies: no `sqlx`, no `tokio`, nothing
that talks to the network or a database. `taskforge-storage` provides two
"adapters" implementing that same trait: `PostgresJobStore` (the real one,
via `sqlx`, using `SELECT ... FOR UPDATE SKIP LOCKED` for atomic job
claiming) and `InMemoryJobStore` (an `Arc<Mutex<HashMap<..>>>`-backed test
double with identical semantics).

Every other crate (`taskforge-worker`, `taskforge-api`, `taskforge-scheduler`)
depends only on `Arc<dyn JobStore>` — never on `PostgresJobStore` directly.
This means:
- `taskforge-worker`'s retry/backoff/panic-isolation logic is fully tested
  against `InMemoryJobStore`, with zero Postgres involved.
- `taskforge-api`'s HTTP surface is fully tested (via `axum`'s in-process
  test utilities) against `InMemoryJobStore` too.
- `PostgresJobStore` itself gets its own, separate, `#[ignore]`d integration
  tests that require a real local Postgres — see
  `taskforge-storage/README.md` for how to run them.
- At runtime, swapping `InMemoryJobStore` for `PostgresJobStore` is a
  one-line change in `main.rs` — the trait boundary is the actual
  architecture, not an afterthought.

A multi-crate Cargo workspace (not one monolithic crate) makes each of
these boundaries a real compiler-enforced boundary, not just a convention:
`taskforge-core` genuinely cannot import `sqlx` by accident, because it's
not a dependency of that crate at all.

## Consequences

- **Positive**: every crate's interesting logic is testable without infra;
  the trait boundary directly demonstrates a real system-design pattern
  (dependency inversion) the learner will recognize from Phase 3-4's own
  framing of the same idea; swapping storage backends later (e.g. adding a
  `RedisJobStore` for a different deployment) is architecturally cheap.
- **Negative**: more upfront ceremony (a trait + two implementations)
  than just calling `sqlx` directly everywhere — a deliberate trade,
  justified here by both the testability constraint and the pedagogical
  value.
- **Follow-up ADRs**: 0002 covers why Postgres (not a message broker) was
  chosen as the queue backend for v1; 0003 covers the job state machine;
  0004 covers the worker's failure-handling model.

## What each crate reinforces from earlier lessons

| Crate | Reinforces |
|---|---|
| `taskforge-core` | enums/pattern matching (Phase 1), traits/generics (Phase 2), custom error types (Phase 2) |
| `taskforge-storage` | Postgres/sqlx (Phase 3), query performance/indexing (Phase 3), the `SKIP LOCKED` pattern from Phase 4's toy queue lesson |
| `taskforge-worker` | concurrency/async (Phase 2), `Arc`/channels (Phase 2), designing for failure/backoff (Phase 4 system design) |
| `taskforge-api` | axum/REST (Phase 3), auth (Phase 3), observability (Phase 4) |
| `taskforge-scheduler` | background jobs (Phase 4) |
| `taskforge-admin-bot` | reuses side-quest 2's `teloxide` skills at production scale |
| `taskforge-cli` | "thin client, thick core" — a system-design lesson in itself |
