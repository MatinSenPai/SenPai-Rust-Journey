# taskforge-worker

The async worker pool: `WorkerPool::new(store)` spawns `concurrency`
independent loops (default 4), each claiming a job via `JobStore::claim_next`,
running it against whichever registered `JobHandler` matches its
`job_type`, and reporting the outcome back — success, retry with
exponential-backoff-plus-jitter, or dead-letter once `max_attempts` is
exhausted. Full design reasoning in
[`../docs/adr/0004-worker-failure-handling.md`](../docs/adr/0004-worker-failure-handling.md).

```rust,ignore
let store: Arc<dyn JobStore> = Arc::new(PostgresJobStore::connect(&database_url).await?);
let pool = WorkerPool::new(store)
    .with_concurrency(8)
    .register(Arc::new(SendEmailHandler::new()))
    .register(Arc::new(GenerateReportHandler::new()));

let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
tokio::spawn(pool.run(shutdown_rx));
// ... on SIGTERM: shutdown_tx.send(true)?; then await the pool's run() future
```

## What's tested here, and how, with zero infrastructure

Every test in `src/lib.rs` runs a real `WorkerPool` against a real
`InMemoryJobStore` (from `taskforge-storage`, brought in only as a
`dev-dependency` — this crate's actual code never depends on any concrete
storage implementation, only the `JobStore` trait) and small
purpose-built `JobHandler`s (`AlwaysSucceeds`, `AlwaysFails`,
`AlwaysPanics`). This proves the real failure-handling behavior end to end:

- a successful job is marked `Succeeded`
- a job whose handler returns `Err` is scheduled for retry (`Retrying`)
- **a job whose handler panics doesn't crash the pool** — the pool keeps
  running, and the job is still scheduled for retry, exactly like an
  ordinary `Err`
- a job with no registered handler for its `job_type` is dead-lettered
  immediately (there's no reason to ever retry it)
- exhausting `max_attempts` dead-letters instead of retrying forever

```sh
cargo test -p taskforge-worker
```

## Known gap (documented, not accidental)

If a worker *process* is killed outright (not just a handler panicking —
the whole process dying), any job it had claimed stays `Running` forever
with nothing to finish it — there's no heartbeat/lease-timeout mechanism to
automatically reclaim it in this version. See ADR-0004's "Consequences"
section. A good stretch extension: add a `claimed_at`/heartbeat column and
have `claim_next` also reclaim `Running` jobs whose heartbeat has gone
stale.
