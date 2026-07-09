# taskforge-scheduler

Recurring jobs: register a `ScheduledJob` (a `job_type`, a fixed `payload`,
and an `interval`), and `Scheduler::run` enqueues it — via the same
`JobStore` every other crate uses — once per interval, forever, until
shutdown.

```rust,ignore
let scheduler = Scheduler::new(store)
    .with_schedule(ScheduledJob::new("cleanup_temp_files", json!({}), Duration::from_secs(3600)))
    .with_schedule(ScheduledJob::new("send_daily_digest", json!({}), Duration::from_secs(86_400)));

let (tx, rx) = tokio::sync::watch::channel(false);
tokio::spawn(scheduler.run(rx));
```

Actually *running* a scheduled job once it's enqueued is entirely
`taskforge-worker`'s job — the scheduler only ever calls `JobStore::enqueue`,
same as any other caller (including `taskforge-api`'s `POST /jobs`). This
crate has zero knowledge of workers, handlers, or how a job eventually gets
processed — another instance of the `JobStore` trait boundary from
[`../docs/adr/0001-architecture-overview.md`](../docs/adr/0001-architecture-overview.md)
doing real work.

## Known limitation (documented, not accidental)

Intervals are fixed `Duration`s, not real cron expressions — there's no
"every day at 9am" or "every Monday" support here. A genuine stretch
extension: swap `ScheduledJob::interval: Duration` for a small cron-parsing
crate (e.g. `cron`) and compute the next fire time from a cron expression
instead of a fixed interval — the rest of `Scheduler::run`'s shape barely
changes.

## Running the tests

```sh
cargo test -p taskforge-scheduler
```

No infrastructure needed — tests run against `InMemoryJobStore` (a
`dev-dependency` only; this crate's actual code never depends on a concrete
storage implementation) and use short intervals (10-15ms) to stay fast.
