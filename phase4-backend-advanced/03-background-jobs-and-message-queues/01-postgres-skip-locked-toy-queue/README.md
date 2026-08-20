# 04.3.1 — Postgres `SKIP LOCKED` toy queue

**This lesson is a direct rehearsal for the capstone.** TaskForge (the
project this whole curriculum builds toward) uses exactly the pattern built
here, at production scale, in
[`capstone-taskforge/taskforge-storage/src/postgres.rs`](../../../capstone-taskforge/taskforge-storage/src/postgres.rs)
— read `claim_next` there once you've finished this lesson and you'll
recognize every line. The reasoning for *why* Postgres (instead of a
message broker) is the queue backend is written up in
[`capstone-taskforge/docs/adr/0002-postgres-backed-queue.md`](../../../capstone-taskforge/docs/adr/0002-postgres-backed-queue.md)
— worth reading now, it's short.

## The problem: many workers, one queue, no double-claims

A job queue's core correctness requirement, whether it's backed by
Postgres, Redis, or a dedicated broker: when N worker processes are all
polling for work at once, each pending job must go to **exactly one**
worker. Two workers both picking up the same "send this email" job means a
duplicate email; the queue picking up nobody's job means work silently
never happens.

The naive approach — `SELECT * FROM jobs WHERE status = 'pending' LIMIT 1`,
then a separate `UPDATE` to mark it claimed — has a race: two workers can
both run the `SELECT` before either runs the `UPDATE`, and both think they
got the same job.

## `FOR UPDATE SKIP LOCKED`

Postgres gives you the fix directly in SQL:

```sql
WITH claimed AS (
    SELECT id FROM toy_jobs
    WHERE status = 'pending'
    ORDER BY id
    FOR UPDATE SKIP LOCKED
    LIMIT 1
)
UPDATE toy_jobs SET status = 'running'
WHERE id IN (SELECT id FROM claimed)
RETURNING id, payload
```

- `FOR UPDATE` takes a row-level lock on whatever the `SELECT` matches,
  inside the current transaction.
- `SKIP LOCKED` is the crucial addition: instead of *waiting* for a
  contended row to unlock (the default `FOR UPDATE` behavior), it silently
  skips any row another transaction already has locked and moves on to the
  next candidate.
- Combined with `LIMIT 1`, "select one claimable job, skipping any another
  worker currently has locked" becomes a single atomic query — no window
  where two workers can both believe they claimed the same row, because
  Postgres itself serializes access to each row.
- Wrapping the `SELECT ... FOR UPDATE SKIP LOCKED` and the `UPDATE` in one
  statement (via the `claimed` CTE) rather than two round-trips closes the
  gap the naive approach had entirely — there's no moment between "found a
  candidate" and "marked it claimed" for another worker to interleave.

This is *the* standard pattern for building a queue on top of a relational
database — you'll see it in Postgres-backed queue libraries across every
language, not just Rust.

## Two implementations, same `Queue` trait

Like the caching lesson, this lesson separates the *logic* (does claiming
correctly hand out at most one job per caller, does `complete` require a
prior claim) from the *backend* (real Postgres vs. an in-memory stand-in),
via a small `Queue` trait:

- **`InMemoryQueue`** (what you implement) is a `Mutex`-guarded `VecDeque` —
  no database at all. It gets the "never double-claim" guarantee from
  Rust's `Mutex` serializing access within *one process*, across many
  `tokio` tasks. This is what every non-`#[ignore]`d test in this lesson
  runs against, including a test that spawns many concurrent tasks racing
  to claim the same batch of jobs and asserts none of them get duplicates —
  the same property `SKIP LOCKED` guarantees for real, just proven at
  smaller scale and without a database.
- **`PostgresQueue`** (given, fully implemented) is the real thing: an
  actual `toy_jobs` table, actual `FOR UPDATE SKIP LOCKED`. Its guarantee
  holds across many separate *processes* (real worker programs, possibly on
  different machines), which a `std::sync::Mutex` fundamentally cannot
  provide — a mutex only ever protects concurrent access within one
  process's memory. That's the entire reason a real queue needs a shared,
  external, transactional store instead of an in-memory lock: the workers
  claiming jobs are usually not all the same process.

## Your task

Open `src/lib.rs`. Implement `InMemoryQueue`'s three `Queue` methods
(`enqueue`, `claim_next`, `complete`). `PostgresQueue` is given, fully
implemented — read it, then compare it side-by-side with
`taskforge-storage/src/postgres.rs`'s `claim_next`.

`PostgresQueue`'s tests need a real local Postgres and are `#[ignore]`d.
Both tests share the same `toy_jobs` table, so each one clears it first —
and both carry `#[serial(p4_03_01_toy_queue_db)]` (from the `serial_test`
crate) so `cargo test`'s default concurrent test execution never runs them
at the same instant against that shared, mutable table:

```sh
# a `taskforge`/`taskforge` role and database, matching the rest of this repo:
sudo service postgresql start   # if not already running
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p4-03-01-postgres-skip-locked-toy-queue -- --ignored
```

## Next

`cargo test -p p4-03-01-postgres-skip-locked-toy-queue`, then
`solution/SOLUTION.md` — but only after a real attempt.
