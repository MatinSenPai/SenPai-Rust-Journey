use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use taskforge_core::{Job, JobId, JobStatus};
use uuid::Uuid;

/// Mirrors the `jobs` table exactly. Deriving `sqlx::FromRow` only maps
/// columns to fields at runtime — it does NOT require a live database at
/// compile time (unlike the `sqlx::query!`/`query_as!` macros, which this
/// crate deliberately avoids — see `README.md`).
#[derive(Debug, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub payload: Value,
    pub status: String,
    pub attempt: Option<i32>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JobRow {
    pub fn into_job(self) -> Job {
        let status = match self.status.as_str() {
            "pending" => JobStatus::Pending,
            "running" => JobStatus::Running,
            "succeeded" => JobStatus::Succeeded,
            "retrying" => JobStatus::Retrying {
                attempt: self.attempt.unwrap_or_default() as u32,
                next_attempt_at: self.next_attempt_at.unwrap_or_else(Utc::now),
            },
            "failed" => JobStatus::Failed {
                error: self.error.clone().unwrap_or_default(),
            },
            "dead_letter" => JobStatus::DeadLetter {
                error: self.error.clone().unwrap_or_default(),
            },
            other => unreachable!("unknown job status stored in the database: {other}"),
        };

        Job {
            id: JobId(self.id),
            job_type: self.job_type,
            payload: self.payload,
            status,
            attempts: self.attempts as u32,
            max_attempts: self.max_attempts as u32,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// The `status` tag string stored for a given `JobStatus`, plus the
/// attempt/next_attempt_at/error columns that go with it. Kept as one
/// function (rather than scattering this mapping across call sites) so
/// there's exactly one place that has to stay in sync with `row.status`'s
/// `match` above.
pub fn status_columns(
    status: &JobStatus,
) -> (
    &'static str,
    Option<i32>,
    Option<DateTime<Utc>>,
    Option<String>,
) {
    match status {
        JobStatus::Pending => ("pending", None, None, None),
        JobStatus::Running => ("running", None, None, None),
        JobStatus::Succeeded => ("succeeded", None, None, None),
        JobStatus::Retrying {
            attempt,
            next_attempt_at,
        } => (
            "retrying",
            Some(*attempt as i32),
            Some(*next_attempt_at),
            None,
        ),
        JobStatus::Failed { error } => ("failed", None, None, Some(error.clone())),
        JobStatus::DeadLetter { error } => ("dead_letter", None, None, Some(error.clone())),
    }
}
