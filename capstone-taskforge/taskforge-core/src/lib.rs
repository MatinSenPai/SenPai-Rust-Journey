//! Domain layer for TaskForge — job types, the job state machine, and the
//! `JobStore`/`JobHandler` ports (traits). Deliberately zero I/O: no
//! `sqlx`, no `tokio` networking, nothing that talks to a database or the
//! network. See `../docs/adr/0001-architecture-overview.md` for why.

mod error;
mod job;
mod store;

pub use error::JobError;
pub use job::{Job, JobId, JobStatus, NewJob};
pub use store::{JobFilter, JobStore};

use async_trait::async_trait;
use serde_json::Value;

/// Implemented by whatever code actually performs a job's work. Registered
/// with a worker pool (`taskforge-worker`) keyed by `job_type()`.
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// The `job_type` this handler processes — must match `Job::job_type`
    /// for jobs it's registered to handle.
    fn job_type(&self) -> &str;

    /// Performs the job's work. Returning `Err` marks the job for retry
    /// (or dead-letter, once `max_attempts` is exhausted) — see
    /// `../docs/adr/0004-worker-failure-handling.md`.
    async fn handle(&self, payload: &Value) -> Result<(), JobError>;
}
