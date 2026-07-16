# Capstone: TaskForge

A production-grade, Postgres-backed background job/task-queue engine — the
Rust equivalent of Sidekiq (Ruby), Celery (Python), or BullMQ (Node). This is
the flagship project of the whole curriculum: not a themed toy, but a
genuinely marketable piece of infrastructure that demonstrates the full
breadth of everything in Phase 0-4 working together.

**Why a job queue, specifically:** almost every real backend eventually needs
"do this work later, reliably, possibly retried" — sending emails,
processing uploads, calling slow third-party APIs, scheduled reports. Companies
pay for (or build in-house) exactly this category of tool. Building one from
scratch — including the failure modes (crashed workers, duplicate delivery,
poison-pill jobs) — is a stronger signal of backend engineering ability than
almost any CRUD app.

## Architecture

A multi-crate Cargo workspace (all members of the repo's single root
workspace, prefixed `taskforge-*`):

| Crate | Responsibility | Key crates used |
|---|---|---|
| `taskforge-core` | Domain types: `Job`, the `JobStatus` state machine (`Pending → Running → {Succeeded, Failed→Retrying, DeadLetter}`), the `JobHandler` trait. No I/O. | `serde`, `thiserror` |
| `taskforge-storage` | Postgres repository: schema/migrations, atomic job claiming via `SELECT ... FOR UPDATE SKIP LOCKED` | `sqlx` |
| `taskforge-worker` | Worker pool: tokio tasks claim jobs, per-job panic isolation, exponential backoff + jitter on retry, graceful shutdown | `tokio`, `tracing` |
| `taskforge-scheduler` | Recurring/cron-style jobs, dead-letter handling | `tokio`, `chrono` |
| `taskforge-api` | REST surface: enqueue/inspect/cancel/list jobs, auth-gated, OpenAPI docs, Prometheus metrics endpoint | `axum`, `tower`, `utoipa`, `metrics-exporter-prometheus` |
| `taskforge-admin-bot` | Telegram bot as an admin/ChatOps client (`/status`, `/retry <id>`, `/pause_queue`) | `teloxide`, calls `taskforge-api` |
| `taskforge-cli` *(stretch)* | `clap`-based CLI hitting the same API — demonstrates "thin client, thick core" | `clap`, `reqwest` |

Each crate reinforces specific earlier lessons — see the mapping in
[`docs/adr/0001-architecture-overview.md`](docs/adr/0001-architecture-overview.md).

## How this capstone works

Every earlier lesson handed you a skeleton with `todo!()`s to fill in. The
capstone is deliberately different, in two halves:

1. **The domain core is already written** — as a complete, tested reference
   codebase, the kind you'd inherit on your first day at a job. Your first
   task isn't to type it; it's to *read* it like a senior engineer:
   understand why it's shaped the way it is, run its tests, and be able to
   explain any decision in it.
2. **The ops layer is yours to build** — the part that turns a well-designed
   library into a system that actually runs, deploys, and survives load.
   That's where the `todo!()`s live now, and it's the half that most
   portfolios are missing.

This mirrors reality: you rarely write a system from a blank file, but you're
constantly expected to take working domain logic and make it *operable*.

### Half 1 — What's already built (study it)

Read the crates in this order — it's also the dependency order, and each has
its own `README.md` plus the Architecture Decision Record that motivated it in
[`docs/adr/`](docs/adr/) (the ADRs are the design narrative — read the ADR
*before* the crate it describes):

1. `taskforge-core` → 2. `taskforge-storage` → 3. `taskforge-worker` →
4. `taskforge-scheduler` → 5. `taskforge-api` → 6. `taskforge-admin-bot` →
7. `taskforge-cli`.

As you read each crate, **run its tests** (`cargo test -p taskforge-core`,
etc.) — they all pass with zero infrastructure (the Postgres-backed tests are
`#[ignore]`d), and reading a test is the fastest way to see the intended
behaviour of the code above it. You should come out of this half able to
answer: how does a job move through its state machine? Why does claiming use
`SELECT … FOR UPDATE SKIP LOCKED`? How does the worker avoid losing a job when
it panics mid-run?

### Half 2 — What you build (the ops layer)

The reference core can be tested, but it can't yet be *run*: three services
are libraries with no entrypoint, there's no way to stand the system up, and
nothing has been put under load. That's your work:

1. **The three service binaries.** `taskforge-api/src/main.rs`,
   `taskforge-worker/src/main.rs`, and `taskforge-scheduler/src/main.rs` each
   ship as a workbook exercise: config-reading and graceful-shutdown wiring
   are written for you (graceful shutdown is the one production pattern no
   earlier lesson covered — study it), and a handful of `todo!()`s have you
   connect the Postgres store, run migrations, and start each service. They
   compile as given; they `panic!` at the `todo!()`s until you finish them.
2. **The container stack.** `docker compose -f docker-compose.yml up --build`
   builds all three binaries (see `Dockerfile`) and wires them to Postgres.
   Until half 2.1 is done, the service containers crash-loop on the `todo!()`
   panics — **the stack staying up is the acceptance test.**
3. **The load test + write-up.** [`loadtest/`](loadtest/) has a `k6` script
   and a set of pointed questions. Run it against your stack, then write the
   "what I'd change at 10× scale" analysis. That write-up is the single best
   interview artifact in this whole repo.

## What "done" looks like

- A crashed worker mid-job doesn't lose or duplicate the job (idempotent
  re-claim — directly exercises Phase 4's distributed-locking module). *This
  is already true in the reference `taskforge-worker` — find the test that
  proves it.*
- `docker compose up --build` brings up the whole system — api, worker,
  scheduler, Postgres — and it **stays** up, once you've finished the three
  `src/main.rs` binaries.
- The `k6` load test in [`loadtest/`](loadtest/) passes its thresholds, and
  you've written the "what I'd change at 10× scale" analysis it prompts for.

Progress against this capstone is tracked in the "Capstone — TaskForge"
section of [`../PROGRESS.md`](../PROGRESS.md).
