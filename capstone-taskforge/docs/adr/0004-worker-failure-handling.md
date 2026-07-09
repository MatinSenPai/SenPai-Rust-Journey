# ADR-0004: Worker failure handling

## Status
Accepted

## Context

Background workers run arbitrary handler code (`JobHandler::handle`)
supplied by whoever registers a job type. That code can panic, return an
error, or hang. A queue engine's entire value proposition rests on handling
all three without losing jobs, corrupting state, or taking down the whole
process — this is exactly the "designing for failure" material from
`phase4-backend-advanced/06-system-design-fundamentals`.

## Decision

- **Panic isolation**: each job runs inside its own `tokio::spawn`ed task.
  A panic inside a spawned task does not unwind into the worker loop or
  crash the process — it's caught as an `Err` on the task's `JoinHandle`,
  which the worker loop treats identically to a handler returning
  `Err(JobError)`: the job is marked `Retrying` (or `DeadLetter` if
  `attempts >= max_attempts`) with the panic message captured as the
  error.
- **Exponential backoff with jitter**: on failure, the next retry delay is
  `base_delay * 2^attempt`, capped at a max delay, with up to ±20% random
  jitter added. Jitter matters at real scale: without it, many jobs that
  failed at the same moment (e.g. a downstream API blip) would all retry
  at exactly the same moment again, creating a synchronized "thundering
  herd" against whatever they're calling — the same failure mode
  `phase4-backend-advanced/01-caching-with-redis` discusses for cache
  stampedes.
- **Dead-lettering**: once `attempts >= max_attempts`, a job moves to
  `DeadLetter` permanently rather than retrying forever — an infinite
  retry loop on a genuinely broken job would silently waste worker
  capacity indefinitely. Dead-lettered jobs are inspectable via
  `taskforge-api` and manually retryable (an admin action, not automatic).
- **Idempotent claiming**: a worker that crashes mid-job (process killed,
  not just the job panicking) leaves that job's row in `Running` with no
  worker left to finish it. `taskforge-storage`'s claim query only selects
  jobs that are `Pending`, or `Retrying` past their `next_attempt_at` — a
  stretch extension (documented in `taskforge-worker/README.md`) is adding
  a heartbeat/lease timeout so a `Running` job whose worker went silent
  gets automatically reclaimed, rather than requiring manual intervention.
  v1 ships without automatic reclaim-on-crash, and says so explicitly
  rather than pretending otherwise.
- **Graceful shutdown**: the worker pool listens for a shutdown signal
  (`tokio::sync::watch` channel) and stops claiming *new* jobs immediately,
  but waits for in-flight jobs to finish (up to a timeout) before exiting —
  so a deploy/restart doesn't abandon jobs mid-execution.

## Consequences

- A single bad job (panicking, erroring, or even hanging past a
  per-job timeout — also configurable) cannot take down the worker
  process or block other jobs from being claimed by other worker tasks.
- Dead-lettering plus manual retry means a human stays in the loop for
  jobs that are *repeatedly* failing, rather than either silently dropping
  them or retrying forever.
- The known gap (no automatic reclaim after a hard worker crash) is
  intentional v1 scope, not an oversight — and is exactly the kind of
  trade-off worth being able to articulate out loud in an interview.
