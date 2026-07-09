//! Recurring jobs: enqueue a fixed job on a fixed interval, forever, until
//! shutdown. Deliberately simple (fixed `Duration` intervals, not real
//! cron-expression parsing) — see `README.md` for the stretch extension.

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use taskforge_core::{JobStore, NewJob};
use tokio::sync::watch;

/// One recurring job definition: enqueue `job_type` with `payload` every
/// `interval`.
pub struct ScheduledJob {
    pub job_type: String,
    pub payload: Value,
    pub interval: Duration,
}

impl ScheduledJob {
    pub fn new(job_type: impl Into<String>, payload: Value, interval: Duration) -> Self {
        ScheduledJob {
            job_type: job_type.into(),
            payload,
            interval,
        }
    }
}

pub struct Scheduler {
    store: Arc<dyn JobStore>,
    schedules: Vec<ScheduledJob>,
}

impl Scheduler {
    pub fn new(store: Arc<dyn JobStore>) -> Self {
        Scheduler {
            store,
            schedules: Vec::new(),
        }
    }

    pub fn with_schedule(mut self, schedule: ScheduledJob) -> Self {
        self.schedules.push(schedule);
        self
    }

    /// Runs every registered schedule concurrently until `shutdown` fires.
    /// Each schedule enqueues its job once per `interval`; a schedule
    /// never blocks on the previous enqueue still being "processed" —
    /// enqueueing is fire-and-forget from the scheduler's point of view,
    /// actually running the job is `taskforge-worker`'s job entirely.
    pub async fn run(self, shutdown: watch::Receiver<bool>) {
        let mut tasks = Vec::with_capacity(self.schedules.len());
        for schedule in self.schedules {
            let store = Arc::clone(&self.store);
            let mut shutdown = shutdown.clone();
            tasks.push(tokio::spawn(async move {
                loop {
                    if *shutdown.borrow() {
                        break;
                    }
                    tokio::select! {
                        _ = tokio::time::sleep(schedule.interval) => {
                            let new_job = NewJob::new(schedule.job_type.clone(), schedule.payload.clone());
                            if let Err(error) = store.enqueue(new_job).await {
                                tracing::error!(?error, job_type = %schedule.job_type, "scheduled enqueue failed");
                            }
                        }
                        _ = shutdown.changed() => {
                            if *shutdown.borrow() {
                                break;
                            }
                        }
                    }
                }
            }));
        }

        for task in tasks {
            let _ = task.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taskforge_core::JobFilter;
    use taskforge_storage::InMemoryJobStore;

    #[tokio::test]
    async fn enqueues_repeatedly_on_its_interval() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let scheduler = Scheduler::new(Arc::clone(&store)).with_schedule(ScheduledJob::new(
            "cleanup_temp_files",
            serde_json::json!({}),
            Duration::from_millis(15),
        ));

        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(scheduler.run(rx));
        tokio::time::sleep(Duration::from_millis(80)).await;
        tx.send(true).unwrap();
        handle.await.unwrap();

        let jobs = store
            .list(JobFilter {
                job_type: Some("cleanup_temp_files".to_string()),
                limit: 100,
            })
            .await
            .unwrap();
        assert!(
            jobs.len() >= 3,
            "expected several enqueues in ~80ms at a 15ms interval, got {}",
            jobs.len()
        );
    }

    #[tokio::test]
    async fn multiple_schedules_run_independently() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let scheduler = Scheduler::new(Arc::clone(&store))
            .with_schedule(ScheduledJob::new(
                "job_a",
                serde_json::json!({}),
                Duration::from_millis(15),
            ))
            .with_schedule(ScheduledJob::new(
                "job_b",
                serde_json::json!({}),
                Duration::from_millis(15),
            ));

        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(scheduler.run(rx));
        tokio::time::sleep(Duration::from_millis(50)).await;
        tx.send(true).unwrap();
        handle.await.unwrap();

        let a = store
            .list(JobFilter {
                job_type: Some("job_a".to_string()),
                limit: 100,
            })
            .await
            .unwrap();
        let b = store
            .list(JobFilter {
                job_type: Some("job_b".to_string()),
                limit: 100,
            })
            .await
            .unwrap();
        assert!(!a.is_empty());
        assert!(!b.is_empty());
    }

    #[tokio::test]
    async fn stops_enqueueing_after_shutdown() {
        let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
        let scheduler = Scheduler::new(Arc::clone(&store)).with_schedule(ScheduledJob::new(
            "job_a",
            serde_json::json!({}),
            Duration::from_millis(10),
        ));

        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(scheduler.run(rx));
        tokio::time::sleep(Duration::from_millis(30)).await;
        tx.send(true).unwrap();
        handle.await.unwrap();

        let count_at_shutdown = store
            .list(JobFilter {
                job_type: Some("job_a".to_string()),
                limit: 1000,
            })
            .await
            .unwrap()
            .len();

        tokio::time::sleep(Duration::from_millis(50)).await;
        let count_later = store
            .list(JobFilter {
                job_type: Some("job_a".to_string()),
                limit: 1000,
            })
            .await
            .unwrap()
            .len();

        assert_eq!(
            count_at_shutdown, count_later,
            "scheduler kept enqueueing after shutdown"
        );
    }
}
