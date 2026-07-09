use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A job's identity. A thin wrapper around `Uuid` rather than a bare
/// `Uuid` field everywhere — this is the same "make illegal states
/// unrepresentable" instinct as the state machine below, just applied to
/// identity: a `JobId` can never accidentally be compared against, say, a
/// `UserId` that also happens to be a `Uuid`, because they're different
/// types even though they wrap the same underlying representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    pub fn new() -> Self {
        JobId(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// See `../docs/adr/0003-job-state-machine.md` for the full reasoning and
/// the valid-transitions diagram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Retrying {
        attempt: u32,
        next_attempt_at: DateTime<Utc>,
    },
    Failed {
        error: String,
    },
    DeadLetter {
        error: String,
    },
}

impl JobStatus {
    /// `Succeeded`, `Failed`, and `DeadLetter` are terminal — a worker will
    /// never claim a job in one of these states again.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Succeeded | JobStatus::Failed { .. } | JobStatus::DeadLetter { .. }
        )
    }

    /// Only `Pending` and cancelled-eligible `Retrying` jobs can be
    /// cancelled by a user via the API — a `Running` job is already being
    /// worked on, and terminal states are, by definition, already done.
    pub fn is_cancellable(&self) -> bool {
        matches!(self, JobStatus::Pending | JobStatus::Retrying { .. })
    }
}

/// What a caller provides to enqueue a new job — deliberately a separate
/// type from `Job` itself (rather than letting callers construct a `Job`
/// directly), so a caller can never accidentally set `status` to something
/// other than the correct initial `Pending`, or fabricate an `id`/
/// `created_at` that didn't come from the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJob {
    pub job_type: String,
    pub payload: Value,
    pub max_attempts: u32,
}

impl NewJob {
    pub fn new(job_type: impl Into<String>, payload: Value) -> Self {
        NewJob {
            job_type: job_type.into(),
            payload,
            max_attempts: 5,
        }
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub job_type: String,
    pub payload: Value,
    pub status: JobStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    /// Constructs a brand-new `Pending` job from a `NewJob` request. Only
    /// `JobStore` implementations call this, right before persisting —
    /// it's how `created_at`/`updated_at`/`id` end up trustworthy.
    pub fn from_new(new_job: NewJob) -> Self {
        let now = Utc::now();
        Job {
            id: JobId::new(),
            job_type: new_job.job_type,
            payload: new_job.payload,
            status: JobStatus::Pending,
            attempts: 0,
            max_attempts: new_job.max_attempts,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_job_starts_pending_with_zero_attempts() {
        let job = Job::from_new(NewJob::new(
            "send_email",
            serde_json::json!({"to": "a@b.com"}),
        ));
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.attempts, 0);
        assert_eq!(job.max_attempts, 5);
    }

    #[test]
    fn with_max_attempts_overrides_default() {
        let new_job = NewJob::new("send_email", serde_json::json!({})).with_max_attempts(1);
        assert_eq!(new_job.max_attempts, 1);
    }

    #[test]
    fn only_terminal_states_report_terminal() {
        assert!(!JobStatus::Pending.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Succeeded.is_terminal());
        assert!(JobStatus::Failed { error: "x".into() }.is_terminal());
        assert!(JobStatus::DeadLetter { error: "x".into() }.is_terminal());
        assert!(!JobStatus::Retrying {
            attempt: 1,
            next_attempt_at: Utc::now()
        }
        .is_terminal());
    }

    #[test]
    fn only_pending_and_retrying_are_cancellable() {
        assert!(JobStatus::Pending.is_cancellable());
        assert!(JobStatus::Retrying {
            attempt: 1,
            next_attempt_at: Utc::now()
        }
        .is_cancellable());
        assert!(!JobStatus::Running.is_cancellable());
        assert!(!JobStatus::Succeeded.is_cancellable());
    }

    #[test]
    fn job_ids_are_unique() {
        assert_ne!(JobId::new(), JobId::new());
    }
}
