//! A small job queue over a `Queue` trait — see `README.md` for the theory.
//! Two implementations: `InMemoryQueue` (what you implement, backs every
//! non-`#[ignore]`d test) and `PostgresQueue` (given, the real
//! `FOR UPDATE SKIP LOCKED` thing — a small-scale rehearsal of
//! `capstone-taskforge/taskforge-storage/src/postgres.rs`).

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::Mutex;

/// Everything that can go wrong talking to a queue backend.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("queue backend error: {0}")]
    Backend(String),
    #[error("job {0} was not claimed (or does not exist), so it cannot be completed")]
    NotClaimed(i64),
}

/// A unit of work: an opaque `payload` string (a real system would use
/// JSON, like `taskforge-core::Job`) plus the id the queue assigned it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: i64,
    pub payload: String,
}

/// A tiny job queue: enqueue a payload, atomically claim the oldest
/// unclaimed job (never handing the same job to two concurrent callers),
/// then mark it complete. Futures are `+ Send` (unlike the caching lesson's
/// `Cache` trait) because this lesson's tests deliberately spawn many
/// concurrent `tokio` tasks racing to claim jobs — `tokio::spawn` requires
/// the spawned future to be `Send`.
pub trait Queue {
    fn enqueue(&self, payload: String) -> impl Future<Output = Result<i64, QueueError>> + Send;

    fn claim_next(&self) -> impl Future<Output = Result<Option<Job>, QueueError>> + Send;

    fn complete(&self, id: i64) -> impl Future<Output = Result<(), QueueError>> + Send;
}

#[derive(Default)]
struct InMemoryQueueState {
    next_id: i64,
    pending: VecDeque<Job>,
    in_progress: HashMap<i64, Job>,
    completed: Vec<i64>,
}

/// An in-memory, `Mutex`-guarded `Queue`. Safe to share across many `tokio`
/// tasks via `Arc<InMemoryQueue>` — the `Mutex` is what gives this the
/// "never double-claim" guarantee within one process, standing in for what
/// `FOR UPDATE SKIP LOCKED` guarantees across many processes for real.
#[derive(Default)]
pub struct InMemoryQueue {
    state: Mutex<InMemoryQueueState>,
}

impl InMemoryQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test/debugging helper: how many jobs have been completed so far.
    pub fn completed_count(&self) -> usize {
        self.state.lock().unwrap().completed.len()
    }
}

impl Queue for InMemoryQueue {
    async fn enqueue(&self, payload: String) -> Result<i64, QueueError> {
        todo!(
            "lock self.state; assign id = state.next_id, then state.next_id += 1; \
             push Job {{ id, payload }} onto state.pending; return Ok(id)"
        )
    }

    async fn claim_next(&self) -> Result<Option<Job>, QueueError> {
        todo!(
            "lock self.state; pop_front() from state.pending; if Some(job), also \
             insert a clone into state.in_progress keyed by job.id before returning \
             Ok(Some(job)); if the queue was empty, return Ok(None)"
        )
    }

    async fn complete(&self, id: i64) -> Result<(), QueueError> {
        todo!(
            "lock self.state; state.in_progress.remove(&id) — if it returns Some(_), \
             push id onto state.completed and return Ok(()); if it returns None, \
             return Err(QueueError::NotClaimed(id)) (nothing was claimed under that id)"
        )
    }
}

/// The real, Postgres-backed `Queue`, given fully implemented so you can
/// see production `FOR UPDATE SKIP LOCKED` code. `#[ignore]`d tests below
/// exercise this against a genuinely live Postgres — see `README.md`.
pub struct PostgresQueue {
    pool: sqlx::PgPool,
}

impl PostgresQueue {
    /// Connects and creates the `toy_jobs` table if it doesn't already
    /// exist. A real system would use `sqlx::migrate!` (see
    /// `taskforge-storage`) — this toy inlines a `CREATE TABLE IF NOT
    /// EXISTS` instead, to keep the lesson to a single file.
    pub async fn connect(database_url: &str) -> Result<Self, QueueError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| QueueError::Backend(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS toy_jobs ( \
                id BIGSERIAL PRIMARY KEY, \
                payload TEXT NOT NULL, \
                status TEXT NOT NULL DEFAULT 'pending', \
                created_at TIMESTAMPTZ NOT NULL DEFAULT now() \
             )",
        )
        .execute(&pool)
        .await
        .map_err(|e| QueueError::Backend(e.to_string()))?;

        Ok(Self { pool })
    }
}

impl Queue for PostgresQueue {
    async fn enqueue(&self, payload: String) -> Result<i64, QueueError> {
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO toy_jobs (payload) VALUES ($1) RETURNING id")
                .bind(payload)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| QueueError::Backend(e.to_string()))?;
        Ok(id)
    }

    async fn claim_next(&self) -> Result<Option<Job>, QueueError> {
        // Same shape as taskforge-storage/src/postgres.rs's claim_next,
        // scaled down: one CTE selects (and locks) one claimable row, the
        // outer UPDATE flips it to 'running' and hands it back, all in one
        // round trip.
        let row: Option<(i64, String)> = sqlx::query_as(
            "WITH claimed AS ( \
                SELECT id FROM toy_jobs \
                WHERE status = 'pending' \
                ORDER BY id \
                FOR UPDATE SKIP LOCKED \
                LIMIT 1 \
             ) \
             UPDATE toy_jobs SET status = 'running' \
             WHERE id IN (SELECT id FROM claimed) \
             RETURNING id, payload",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::Backend(e.to_string()))?;

        Ok(row.map(|(id, payload)| Job { id, payload }))
    }

    async fn complete(&self, id: i64) -> Result<(), QueueError> {
        let result =
            sqlx::query("UPDATE toy_jobs SET status = 'done' WHERE id = $1 AND status = 'running'")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| QueueError::Backend(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(QueueError::NotClaimed(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn enqueue_then_claim_returns_the_job() {
        let queue = InMemoryQueue::new();
        let id = queue.enqueue("send-email".to_string()).await.unwrap();

        let claimed = queue.claim_next().await.unwrap().unwrap();
        assert_eq!(claimed.id, id);
        assert_eq!(claimed.payload, "send-email");
    }

    #[tokio::test]
    async fn claim_on_empty_queue_returns_none() {
        let queue = InMemoryQueue::new();
        assert_eq!(queue.claim_next().await.unwrap(), None);
    }

    #[tokio::test]
    async fn claims_are_returned_in_fifo_order() {
        let queue = InMemoryQueue::new();
        queue.enqueue("first".to_string()).await.unwrap();
        queue.enqueue("second".to_string()).await.unwrap();

        let first = queue.claim_next().await.unwrap().unwrap();
        let second = queue.claim_next().await.unwrap().unwrap();

        assert_eq!(first.payload, "first");
        assert_eq!(second.payload, "second");
    }

    #[tokio::test]
    async fn complete_requires_a_prior_claim() {
        let queue = InMemoryQueue::new();
        let id = queue.enqueue("payload".to_string()).await.unwrap();

        // Not claimed yet — completing should fail.
        assert!(matches!(
            queue.complete(id).await,
            Err(QueueError::NotClaimed(_))
        ));

        queue.claim_next().await.unwrap();
        queue.complete(id).await.unwrap();
        assert_eq!(queue.completed_count(), 1);

        // Completing the same job twice should also fail — it's no longer
        // "in progress" after the first `complete`.
        assert!(matches!(
            queue.complete(id).await,
            Err(QueueError::NotClaimed(_))
        ));
    }

    /// The property `FOR UPDATE SKIP LOCKED` guarantees for real, at
    /// process scale, proven here at task scale: many concurrent callers
    /// racing to claim jobs from the same queue never see a duplicate.
    #[tokio::test]
    async fn concurrent_claims_never_double_claim_a_job() {
        let queue = Arc::new(InMemoryQueue::new());
        let job_count = 50;
        for i in 0..job_count {
            queue.enqueue(format!("job-{i}")).await.unwrap();
        }

        let mut handles = Vec::new();
        for _ in 0..10 {
            let queue = Arc::clone(&queue);
            handles.push(tokio::spawn(async move {
                let mut claimed = Vec::new();
                while let Some(job) = queue.claim_next().await.unwrap() {
                    claimed.push(job.id);
                }
                claimed
            }));
        }

        let mut all_claimed = Vec::new();
        for handle in handles {
            all_claimed.extend(handle.await.unwrap());
        }

        all_claimed.sort_unstable();
        let mut deduped = all_claimed.clone();
        deduped.dedup();
        assert_eq!(
            all_claimed.len(),
            deduped.len(),
            "no job should be claimed by more than one task"
        );
        assert_eq!(
            all_claimed.len(),
            job_count,
            "every job should be claimed exactly once"
        );
    }

    /// Needs a real local Postgres. Start one first (see README.md), then:
    ///   DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
    ///     cargo test -p p4-03-01-postgres-skip-locked-toy-queue -- --ignored --test-threads=1
    ///
    /// Both `#[ignore]`d tests below share the same `toy_jobs` table, so
    /// each one clears it first — otherwise leftover rows from a previous
    /// run (or from the *other* ignored test, if run in parallel) could
    /// make `claim_next`'s FIFO ordering hand back someone else's job
    /// instead of the one this test just enqueued. `cargo test -- --ignored --test-threads=1`
    /// runs tests in parallel by default; without this reset these two
    /// tests would be flaky against each other.
    #[tokio::test]
    #[ignore]
    async fn postgres_queue_round_trips_against_a_live_postgres() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let queue = PostgresQueue::connect(&database_url).await.unwrap();
        sqlx::query("DELETE FROM toy_jobs")
            .execute(&queue.pool)
            .await
            .unwrap();

        let id = queue
            .enqueue("integration-test-payload".to_string())
            .await
            .unwrap();
        let claimed = queue.claim_next().await.unwrap().unwrap();
        assert_eq!(claimed.id, id);
        assert_eq!(claimed.payload, "integration-test-payload");

        queue.complete(id).await.unwrap();

        // Completing an already-completed job should fail, same contract
        // as InMemoryQueue.
        assert!(matches!(
            queue.complete(id).await,
            Err(QueueError::NotClaimed(_))
        ));
    }

    /// Same "never double-claim" property as the in-memory test above, but
    /// against real Postgres with real concurrent connections — this is
    /// what actually proves `FOR UPDATE SKIP LOCKED` works, not just the
    /// `Mutex`-based simulation.
    #[tokio::test]
    #[ignore]
    async fn postgres_concurrent_claims_never_double_claim_a_job() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let queue = Arc::new(PostgresQueue::connect(&database_url).await.unwrap());
        sqlx::query("DELETE FROM toy_jobs")
            .execute(&queue.pool)
            .await
            .unwrap();

        let job_count = 20;
        for i in 0..job_count {
            queue.enqueue(format!("concurrent-job-{i}")).await.unwrap();
        }

        let mut handles = Vec::new();
        for _ in 0..5 {
            let queue = Arc::clone(&queue);
            handles.push(tokio::spawn(async move {
                let mut claimed = Vec::new();
                while let Some(job) = queue.claim_next().await.unwrap() {
                    claimed.push(job.id);
                }
                claimed
            }));
        }

        let mut all_claimed = Vec::new();
        for handle in handles {
            all_claimed.extend(handle.await.unwrap());
        }
        all_claimed.sort_unstable();
        let mut deduped = all_claimed.clone();
        deduped.dedup();
        assert_eq!(all_claimed.len(), deduped.len());
    }
}
