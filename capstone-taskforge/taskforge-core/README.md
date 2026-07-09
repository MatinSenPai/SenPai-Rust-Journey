# taskforge-core

The domain layer. Defines what a job *is* (`Job`, `JobId`, `JobStatus`,
`NewJob`), the rules for how a job's status may change (see
[`../docs/adr/0003-job-state-machine.md`](../docs/adr/0003-job-state-machine.md)),
and two traits other crates depend on instead of depending on each other
directly:

- **`JobStore`** — the storage port. `taskforge-storage` implements it
  twice: once for real (Postgres), once as an in-memory test double. See
  [`../docs/adr/0001-architecture-overview.md`](../docs/adr/0001-architecture-overview.md).
- **`JobHandler`** — implemented by whatever code actually performs a
  job's work; registered with a worker pool in `taskforge-worker`, keyed
  by `job_type()`.

This crate has **zero I/O dependencies** — no `sqlx`, no `tokio`
networking. That's deliberate, not incidental: it's what makes every other
crate's tests able to run against `InMemoryJobStore` with no database, no
Docker, no network, anywhere.

## Running the tests

```sh
cargo test -p taskforge-core
```

No setup needed — this crate's tests are pure logic (the state machine's
`is_terminal`/`is_cancellable` rules, `Job` construction from `NewJob`,
`JobId` uniqueness).
