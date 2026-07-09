//! The async worker pool: claims jobs from a `taskforge_core::JobStore`,
//! runs them against registered `JobHandler`s, and reports outcomes back
//! (success, retry-with-backoff, or dead-letter). See
//! `../docs/adr/0004-worker-failure-handling.md`.

mod backoff;
mod pool;

pub use backoff::{compute_backoff, BackoffConfig};
pub use pool::WorkerPool;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use taskforge_core::{JobError, JobHandler, JobStatus, JobStore, NewJob};
    use taskforge_storage::InMemoryJobStore;
    use tokio::sync::watch;

    struct AlwaysSucceeds {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl JobHandler for AlwaysSucceeds {
        fn job_type(&self) -> &str {
            "always_succeeds"
        }
        async fn handle(&self, _payload: &Value) -> Result<(), JobError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct AlwaysFails;

    #[async_trait]
    impl JobHandler for AlwaysFails {
        fn job_type(&self) -> &str {
            "always_fails"
        }
        async fn handle(&self, _payload: &Value) -> Result<(), JobError> {
            Err(JobError::HandlerFailed("simulated failure".to_string()))
        }
    }

    struct AlwaysPanics;

    #[async_trait]
    impl JobHandler for AlwaysPanics {
        fn job_type(&self) -> &str {
            "always_panics"
        }
        async fn handle(&self, _payload: &Value) -> Result<(), JobError> {
            panic!("simulated handler panic");
        }
    }

    /// Runs `pool` in the background, waits a bit for it to make progress,
    /// then signals shutdown and waits for it to fully stop.
    async fn run_briefly(pool: WorkerPool, wait: Duration) {
        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(pool.run(rx));
        tokio::time::sleep(wait).await;
        tx.send(true).unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn successful_job_is_marked_succeeded() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let job = store
            .enqueue(NewJob::new("always_succeeds", serde_json::json!({})))
            .await
            .unwrap();

        let pool = WorkerPool::new(Arc::clone(&store))
            .with_poll_interval(Duration::from_millis(10))
            .register(Arc::new(AlwaysSucceeds {
                calls: Arc::clone(&calls),
            }));
        run_briefly(pool, Duration::from_millis(100)).await;

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Succeeded);
    }

    #[tokio::test]
    async fn failed_job_is_scheduled_for_retry() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let job = store
            .enqueue(NewJob::new("always_fails", serde_json::json!({})).with_max_attempts(5))
            .await
            .unwrap();

        let pool = WorkerPool::new(Arc::clone(&store))
            .with_poll_interval(Duration::from_millis(10))
            .register(Arc::new(AlwaysFails));
        run_briefly(pool, Duration::from_millis(100)).await;

        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert!(
            matches!(fetched.status, JobStatus::Retrying { attempt: 1, .. }),
            "expected Retrying{{attempt: 1}}, got {:?}",
            fetched.status
        );
    }

    #[tokio::test]
    async fn panicking_handler_does_not_crash_the_pool_and_still_schedules_retry() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let job = store
            .enqueue(NewJob::new("always_panics", serde_json::json!({})).with_max_attempts(5))
            .await
            .unwrap();

        let pool = WorkerPool::new(Arc::clone(&store))
            .with_poll_interval(Duration::from_millis(10))
            .register(Arc::new(AlwaysPanics));
        run_briefly(pool, Duration::from_millis(100)).await;

        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert!(matches!(fetched.status, JobStatus::Retrying { .. }));
    }

    #[tokio::test]
    async fn job_with_no_registered_handler_is_dead_lettered_immediately() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let job = store
            .enqueue(NewJob::new("nobody_handles_this", serde_json::json!({})))
            .await
            .unwrap();

        let pool =
            WorkerPool::new(Arc::clone(&store)).with_poll_interval(Duration::from_millis(10));
        run_briefly(pool, Duration::from_millis(50)).await;

        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert!(matches!(fetched.status, JobStatus::DeadLetter { .. }));
    }

    #[tokio::test]
    async fn exhausting_max_attempts_eventually_dead_letters() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let job = store
            .enqueue(NewJob::new("always_fails", serde_json::json!({})).with_max_attempts(1))
            .await
            .unwrap();

        let pool = WorkerPool::new(Arc::clone(&store))
            .with_poll_interval(Duration::from_millis(10))
            .register(Arc::new(AlwaysFails));
        run_briefly(pool, Duration::from_millis(100)).await;

        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert!(matches!(fetched.status, JobStatus::DeadLetter { .. }));
    }
}
