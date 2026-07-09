use crate::{Job, JobError, JobId, NewJob};
use async_trait::async_trait;

/// Optional filters for listing jobs via `JobStore::list`.
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    pub job_type: Option<String>,
    pub limit: usize,
}

/// The storage "port" (see `../docs/adr/0001-architecture-overview.md`).
/// `taskforge-storage` provides two implementations: `PostgresJobStore`
/// (real) and `InMemoryJobStore` (test double, identical semantics).
/// Every other crate depends on `Arc<dyn JobStore>`, never on a concrete
/// implementation directly.
#[async_trait]
pub trait JobStore: Send + Sync {
    /// Persists a new job in `Pending` status and returns it.
    async fn enqueue(&self, new_job: NewJob) -> Result<Job, JobError>;

    /// Atomically claims the next available job — either `Pending`, or
    /// `Retrying` whose `next_attempt_at` has passed — and marks it
    /// `Running`. Returns `None` if nothing is claimable right now. Must
    /// be safe for many workers to call concurrently without two workers
    /// ever claiming the same job (see `taskforge-storage`'s
    /// `SELECT ... FOR UPDATE SKIP LOCKED` implementation).
    async fn claim_next(&self) -> Result<Option<Job>, JobError>;

    /// Marks a `Running` job `Succeeded`.
    async fn mark_succeeded(&self, id: JobId) -> Result<(), JobError>;

    /// Marks a `Running` job either `Retrying` (with the given
    /// `next_attempt_at`) or `DeadLetter`, depending on whether
    /// `attempts` has reached `max_attempts` — see
    /// `../docs/adr/0004-worker-failure-handling.md`.
    async fn mark_failed(
        &self,
        id: JobId,
        error: String,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), JobError>;

    /// Cancels a `Pending` or `Retrying` job. Errors with
    /// `JobError::NotCancellable` for any other status.
    async fn cancel(&self, id: JobId) -> Result<(), JobError>;

    /// Fetches a single job by id.
    async fn get(&self, id: JobId) -> Result<Option<Job>, JobError>;

    /// Lists jobs, optionally filtered by `job_type`, newest first, capped
    /// at `filter.limit` (0 means "use a sensible default").
    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>, JobError>;
}
