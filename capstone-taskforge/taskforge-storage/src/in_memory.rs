use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use taskforge_core::{Job, JobError, JobFilter, JobId, JobStatus, JobStore, NewJob};

/// A `JobStore` implementation with no external dependencies at all — an
/// `Arc<Mutex<HashMap<..>>>` — used throughout this workspace's test
/// suites (`taskforge-worker`, `taskforge-api`) so their logic is fully
/// verified without a live database anywhere. See
/// `../docs/adr/0001-architecture-overview.md`.
///
/// Semantics are meant to match `PostgresJobStore` exactly: same claiming
/// rules (`Pending`, or `Retrying` past `next_attempt_at`), same
/// retry/dead-letter transition on failure, same cancellation rules. The
/// one real difference: `claim_next` here takes a `std::sync::Mutex` lock
/// for its whole body, so "only one job claimed at a time" is enforced by
/// the lock itself rather than a database-level `SKIP LOCKED` — fine for a
/// test double, not how you'd want a real high-throughput store to work.
#[derive(Default)]
pub struct InMemoryJobStore {
    jobs: Mutex<HashMap<JobId, Job>>,
}

impl InMemoryJobStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl JobStore for InMemoryJobStore {
    async fn enqueue(&self, new_job: NewJob) -> Result<Job, JobError> {
        let job = Job::from_new(new_job);
        self.jobs.lock().unwrap().insert(job.id, job.clone());
        Ok(job)
    }

    async fn claim_next(&self) -> Result<Option<Job>, JobError> {
        let mut jobs = self.jobs.lock().unwrap();
        let now = Utc::now();

        let claimable_id = jobs
            .values()
            .filter(|j| match &j.status {
                JobStatus::Pending => true,
                JobStatus::Retrying {
                    next_attempt_at, ..
                } => *next_attempt_at <= now,
                _ => false,
            })
            .min_by_key(|j| j.created_at)
            .map(|j| j.id);

        let Some(id) = claimable_id else {
            return Ok(None);
        };

        let job = jobs.get_mut(&id).expect("id came from this same map");
        job.status = JobStatus::Running;
        job.updated_at = now;
        Ok(Some(job.clone()))
    }

    async fn mark_succeeded(&self, id: JobId) -> Result<(), JobError> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs
            .get_mut(&id)
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;
        job.status = JobStatus::Succeeded;
        job.updated_at = Utc::now();
        Ok(())
    }

    async fn mark_failed(
        &self,
        id: JobId,
        error: String,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), JobError> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs
            .get_mut(&id)
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;

        job.attempts += 1;
        job.status = match next_attempt_at {
            Some(next_attempt_at) if job.attempts < job.max_attempts => JobStatus::Retrying {
                attempt: job.attempts,
                next_attempt_at,
            },
            _ => JobStatus::DeadLetter { error },
        };
        job.updated_at = Utc::now();
        Ok(())
    }

    async fn cancel(&self, id: JobId) -> Result<(), JobError> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs
            .get_mut(&id)
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;

        if !job.status.is_cancellable() {
            return Err(JobError::NotCancellable(format!("{:?}", job.status)));
        }
        job.status = JobStatus::Failed {
            error: "cancelled by user".to_string(),
        };
        job.updated_at = Utc::now();
        Ok(())
    }

    async fn get(&self, id: JobId) -> Result<Option<Job>, JobError> {
        Ok(self.jobs.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>, JobError> {
        let limit = if filter.limit == 0 { 50 } else { filter.limit };
        let jobs = self.jobs.lock().unwrap();

        let mut matching: Vec<Job> = jobs
            .values()
            .filter(|j| {
                filter
                    .job_type
                    .as_ref()
                    .map(|t| &j.job_type == t)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        matching.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        matching.truncate(limit);
        Ok(matching)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn enqueue_then_claim_marks_running() {
        let store = InMemoryJobStore::new();
        let job = store
            .enqueue(NewJob::new("send_email", serde_json::json!({})))
            .await
            .unwrap();
        assert_eq!(job.status, JobStatus::Pending);

        let claimed = store.claim_next().await.unwrap().unwrap();
        assert_eq!(claimed.id, job.id);
        assert_eq!(claimed.status, JobStatus::Running);

        // Already claimed — nothing else to claim.
        assert!(store.claim_next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn claims_oldest_pending_job_first() {
        let store = InMemoryJobStore::new();
        let first = store
            .enqueue(NewJob::new("a", serde_json::json!({})))
            .await
            .unwrap();
        let _second = store
            .enqueue(NewJob::new("b", serde_json::json!({})))
            .await
            .unwrap();

        let claimed = store.claim_next().await.unwrap().unwrap();
        assert_eq!(claimed.id, first.id);
    }

    #[tokio::test]
    async fn retrying_job_not_claimable_until_next_attempt_at() {
        let store = InMemoryJobStore::new();
        let job = store
            .enqueue(NewJob::new("a", serde_json::json!({})))
            .await
            .unwrap();
        store.claim_next().await.unwrap();
        store
            .mark_failed(
                job.id,
                "boom".to_string(),
                Some(Utc::now() + chrono::Duration::hours(1)),
            )
            .await
            .unwrap();

        assert!(store.claim_next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn exhausting_max_attempts_dead_letters_instead_of_retrying() {
        let store = InMemoryJobStore::new();
        let job = store
            .enqueue(NewJob::new("a", serde_json::json!({})).with_max_attempts(1))
            .await
            .unwrap();
        store.claim_next().await.unwrap();
        store
            .mark_failed(job.id, "boom".to_string(), Some(Utc::now()))
            .await
            .unwrap();

        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert!(matches!(fetched.status, JobStatus::DeadLetter { .. }));
    }

    #[tokio::test]
    async fn cancel_rejects_running_jobs() {
        let store = InMemoryJobStore::new();
        let job = store
            .enqueue(NewJob::new("a", serde_json::json!({})))
            .await
            .unwrap();
        store.claim_next().await.unwrap();

        let result = store.cancel(job.id).await;
        assert!(matches!(result, Err(JobError::NotCancellable(_))));
    }

    #[tokio::test]
    async fn concurrent_claims_never_double_claim() {
        let store = Arc::new(InMemoryJobStore::new());
        for i in 0..20 {
            store
                .enqueue(NewJob::new("a", serde_json::json!({ "i": i })))
                .await
                .unwrap();
        }

        let mut handles = Vec::new();
        for _ in 0..20 {
            let store = Arc::clone(&store);
            handles.push(tokio::spawn(
                async move { store.claim_next().await.unwrap() },
            ));
        }

        let mut claimed_ids = std::collections::HashSet::new();
        for handle in handles {
            if let Some(job) = handle.await.unwrap() {
                assert!(
                    claimed_ids.insert(job.id),
                    "the same job was claimed twice — claim_next is not safe for concurrent workers"
                );
            }
        }
        assert_eq!(claimed_ids.len(), 20);
    }
}
