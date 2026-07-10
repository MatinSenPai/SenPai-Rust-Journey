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

## Building it

Architecture Decision Records in [`docs/adr/`](docs/adr/) are written
**before** each crate, in order — read them in sequence, they're the actual
design narrative, not just the README. Build order: `taskforge-core` →
`taskforge-storage` → `taskforge-worker` → `taskforge-scheduler` →
`taskforge-api` → `taskforge-admin-bot` → `taskforge-cli`.

`docker-compose.yml` brings up Postgres (and Redis, if the rate-limiting
lesson's ideas get folded in) for local development.

## What "done" looks like

- A crashed worker mid-job doesn't lose or duplicate the job (idempotent
  re-claim — directly exercises Phase 4's distributed-locking module).
- `docker-compose up` brings up the whole system from a clean checkout.
- A short load test (e.g. `k6` or `drill`) plus a closing "what I'd change at
  10x scale" write-up in this README — a genuine system-design-maturity
  artifact, and good interview/portfolio material on its own.

Progress against this capstone is tracked in the "Capstone — TaskForge"
section of [`../PROGRESS.md`](../PROGRESS.md).
