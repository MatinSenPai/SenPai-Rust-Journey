use crate::backoff::{compute_backoff, BackoffConfig};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use taskforge_core::{Job, JobHandler, JobStore};
use tokio::sync::watch;

/// A pool of `concurrency` async workers, each independently claiming and
/// running jobs from `store` until told to shut down. See
/// `../docs/adr/0004-worker-failure-handling.md` for the failure-handling
/// design this implements (panic isolation, backoff+jitter, dead-lettering,
/// graceful shutdown).
pub struct WorkerPool {
    store: Arc<dyn JobStore>,
    handlers: Arc<HashMap<String, Arc<dyn JobHandler>>>,
    concurrency: usize,
    backoff: BackoffConfig,
    poll_interval: Duration,
}

impl WorkerPool {
    pub fn new(store: Arc<dyn JobStore>) -> Self {
        WorkerPool {
            store,
            handlers: Arc::new(HashMap::new()),
            concurrency: 4,
            backoff: BackoffConfig::default(),
            poll_interval: Duration::from_millis(200),
        }
    }

    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn with_backoff(mut self, backoff: BackoffConfig) -> Self {
        self.backoff = backoff;
        self
    }

    /// Registers a handler for the job type it declares via
    /// `JobHandler::job_type()`. Registering two handlers for the same
    /// job type silently keeps the last one — call this while setting up,
    /// not dynamically at runtime.
    pub fn register(mut self, handler: Arc<dyn JobHandler>) -> Self {
        Arc::make_mut(&mut self.handlers).insert(handler.job_type().to_string(), handler);
        self
    }

    /// Runs `self.concurrency` worker loops until `shutdown` carries
    /// `true`. Each loop stops claiming *new* jobs as soon as shutdown is
    /// signaled, but always finishes whatever job it's currently running
    /// first — this function only returns once every in-flight job has
    /// completed.
    pub async fn run(self, shutdown: watch::Receiver<bool>) {
        let mut tasks = Vec::with_capacity(self.concurrency);
        for _ in 0..self.concurrency {
            let store = Arc::clone(&self.store);
            let handlers = Arc::clone(&self.handlers);
            let mut shutdown = shutdown.clone();
            let backoff = self.backoff.clone();
            let poll_interval = self.poll_interval;

            tasks.push(tokio::spawn(async move {
                loop {
                    if *shutdown.borrow() {
                        break;
                    }
                    match store.claim_next().await {
                        Ok(Some(job)) => {
                            run_one_job(&store, &handlers, job, &backoff).await;
                        }
                        Ok(None) => {
                            tokio::select! {
                                _ = tokio::time::sleep(poll_interval) => {}
                                _ = shutdown.changed() => {
                                    if *shutdown.borrow() {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            tracing::error!(?error, "claim_next failed; backing off");
                            tokio::time::sleep(poll_interval).await;
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

/// Runs one job in its own spawned task (so a panic inside the handler
/// can't unwind into the worker loop — see ADR-0004), then reports the
/// outcome back to `store`.
async fn run_one_job(
    store: &Arc<dyn JobStore>,
    handlers: &HashMap<String, Arc<dyn JobHandler>>,
    job: Job,
    backoff: &BackoffConfig,
) {
    let Some(handler) = handlers.get(&job.job_type).cloned() else {
        let _ = store
            .mark_failed(
                job.id,
                format!("no handler registered for job_type \"{}\"", job.job_type),
                None, // no handler ever will exist for this job type — dead-letter immediately
            )
            .await;
        return;
    };

    let payload = job.payload.clone();
    let outcome = tokio::spawn(async move { handler.handle(&payload).await }).await;

    match outcome {
        Ok(Ok(())) => {
            let _ = store.mark_succeeded(job.id).await;
        }
        Ok(Err(error)) => {
            let next_attempt_at =
                Utc::now() + to_chrono_duration(compute_backoff(job.attempts + 1, backoff));
            let _ = store
                .mark_failed(job.id, error.to_string(), Some(next_attempt_at))
                .await;
        }
        Err(join_error) => {
            let message = if join_error.is_panic() {
                "handler panicked".to_string()
            } else {
                "handler task was cancelled".to_string()
            };
            let next_attempt_at =
                Utc::now() + to_chrono_duration(compute_backoff(job.attempts + 1, backoff));
            let _ = store
                .mark_failed(job.id, message, Some(next_attempt_at))
                .await;
        }
    }
}

fn to_chrono_duration(d: Duration) -> chrono::Duration {
    chrono::Duration::from_std(d).unwrap_or(chrono::Duration::seconds(1))
}
