# Solution

```rust
impl Queue for InMemoryQueue {
    async fn enqueue(&self, payload: String) -> Result<i64, QueueError> {
        let mut state = self.state.lock().unwrap();
        let id = state.next_id;
        state.next_id += 1;
        state.pending.push_back(Job { id, payload });
        Ok(id)
    }

    async fn claim_next(&self) -> Result<Option<Job>, QueueError> {
        let mut state = self.state.lock().unwrap();
        let Some(job) = state.pending.pop_front() else {
            return Ok(None);
        };
        state.in_progress.insert(job.id, job.clone());
        Ok(Some(job))
    }

    async fn complete(&self, id: i64) -> Result<(), QueueError> {
        let mut state = self.state.lock().unwrap();
        if state.in_progress.remove(&id).is_some() {
            state.completed.push(id);
            Ok(())
        } else {
            Err(QueueError::NotClaimed(id))
        }
    }
}
```

Each method takes the same `Mutex<InMemoryQueueState>` lock for its entire
body — the whole "never double-claim" guarantee for `InMemoryQueue` boils
down to that one lock: `claim_next` pops from `pending` and inserts into
`in_progress` atomically (as far as any other caller can observe), so two
concurrent tasks can never both pop the same front-of-queue job. `complete`
requires the job to actually be in `in_progress` first — completing a job
that was never claimed (or was already completed once) is a caller error,
reported as `QueueError::NotClaimed` rather than silently ignored.

## Why `PostgresQueue` needed no `Mutex` at all

The in-memory version's safety comes entirely from one process's
`std::sync::Mutex`. `PostgresQueue::claim_next` gets the equivalent
guarantee from the database itself:

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

`FOR UPDATE SKIP LOCKED` locks the one row it selects (so no other
transaction can also select it) and, critically, skips any row *already*
locked by a concurrent transaction instead of blocking on it — two workers
calling `claim_next` at the same instant each get a different row, with
zero coordination beyond both of them talking to the same Postgres. This is
the property that scales across separate **processes** (real, independent
worker programs, possibly on different machines), which is exactly what a
production job queue needs and what a `Mutex` — scoped to one process's
memory — structurally cannot provide. Compare this query to
`taskforge-storage/src/postgres.rs`'s `claim_next`: same shape, same
`FOR UPDATE SKIP LOCKED` CTE-then-`UPDATE` pattern, just with more columns
(status enum variants, retry bookkeeping) once it's a real job queue rather
than a three-status toy.

`complete`'s `WHERE id = $1 AND status = 'running'` plus checking
`rows_affected() == 0` is the Postgres-side version of the same "must have
been claimed first" contract `InMemoryQueue` enforces via `in_progress`
membership — both implementations refuse to let you complete a job that
isn't currently claimed, keeping the two `Queue` implementations
behaviorally identical from a caller's point of view, exactly as the
`JobStore` trait boundary in the capstone (`docs/adr/0001`) is designed to
guarantee.

## Why both `#[ignore]`d tests need `#[serial(p4_03_01_toy_queue_db)]`

`cargo test` runs different `#[test]` functions concurrently by default,
and both Postgres tests here share one live `toy_jobs` table — one of them
enqueues 20 jobs and claims them concurrently from 5 tasks; if the other
test's own `DELETE FROM toy_jobs` reset happened to land mid-flight, rows
either test expected to see could vanish out from under it. This isn't
hypothetical: it's a race this repo's own test suite hit while being
verified, only under default parallel execution, never with
`--test-threads=1`. `#[serial]` needs the *same* tag on *both* tests
(not just one) because it only prevents two same-tagged tests from
overlapping — a test without the tag is invisible to it and can still run
concurrently with a tagged one, so one unprotected test is enough to leave
the shared table's mutation still racy.
