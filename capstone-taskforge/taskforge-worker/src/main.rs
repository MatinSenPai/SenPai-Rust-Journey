//! Service binary for `taskforge-worker` — **this file is your exercise**.
//!
//! The library half (the `WorkerPool` in `src/pool.rs`, backoff in
//! `src/backoff.rs`) is finished reference code with its own test suite.
//! This binary's job is the ops layer: read config, connect to the real
//! Postgres store, register handlers, run the pool, and — the interesting
//! part — shut it down *cleanly*, so a `docker stop` never kills a job
//! halfway through.
//!
//! Provided for you:
//! - tracing setup, [`Config::from_env`], [`shutdown_signal`]
//! - [`SendEmail`], a demo `JobHandler`, so the stack visibly processes
//!   work end-to-end (the k6 script in `../loadtest/load.js` and the demo
//!   schedule in `taskforge-scheduler`'s binary both enqueue exactly this
//!   `job_type`)
//!
//! You fill in the two `todo!()`s: [`connect_store`] and [`run_pool`].
//! When both are done:
//!
//! ```sh
//! docker compose -f ../docker-compose.yml up -d postgres
//! DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
//!   cargo run -p taskforge-worker
//! ```
//!
//! …then enqueue a `send_email` job through the API (or the k6 load test)
//! and watch this process log it flowing Pending → Running → Succeeded.

use async_trait::async_trait;
use serde_json::Value;
use std::future::Future;
use std::sync::Arc;
use taskforge_core::{JobError, JobHandler, JobStore};

/// Everything this service reads from the environment, validated once at
/// startup.
struct Config {
    database_url: String,
    /// How many concurrent worker loops to run. More is not always
    /// better: each in-flight job holds a Postgres connection while it
    /// claims/reports, and the pool behind `PostgresJobStore` is capped
    /// at 10 connections — a question the load test will make concrete.
    concurrency: usize,
}

impl Config {
    fn from_env() -> Result<Config, String> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "required env var DATABASE_URL is not set".to_string())?;
        let concurrency = match std::env::var("WORKER_CONCURRENCY") {
            Ok(raw) => raw.parse::<usize>().map_err(|_| {
                format!("WORKER_CONCURRENCY must be a positive integer, got {raw:?}")
            })?,
            Err(_) => 4,
        };
        Ok(Config {
            database_url,
            concurrency,
        })
    }
}

/// A demo handler so the stack does something observable: it "sends" the
/// email by logging the payload and sleeping briefly (a stand-in for real
/// I/O — an SMTP call, an HTTP request to a mail provider…). Real
/// deployments would register one handler per job type here.
struct SendEmail;

#[async_trait]
impl JobHandler for SendEmail {
    fn job_type(&self) -> &str {
        "send_email"
    }

    async fn handle(&self, payload: &Value) -> Result<(), JobError> {
        tracing::info!(%payload, "pretending to send an email");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }
}

/// Resolves when the process is asked to stop: Ctrl-C (SIGINT) anywhere,
/// plus SIGTERM on Unix — SIGTERM is what `docker stop` actually sends.
/// See the longer discussion in `taskforge-api/src/main.rs`.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received; draining in-flight jobs");
}

/// ── Your step 1: Postgres pool + migrations + store ─────────────────────
///
/// Identical to the api binary's step 1 (each service connects on its
/// own): `taskforge_storage::PostgresJobStore::connect(database_url)`
/// builds the `PgPool`, runs `taskforge-storage/migrations/`, and wraps
/// the pool. Migrations are idempotent and advisory-locked, so api and
/// worker racing at startup is fine.
async fn connect_store(
    database_url: &str,
) -> Result<Arc<dyn JobStore>, Box<dyn std::error::Error>> {
    // Keeps `cargo clippy -- -D warnings` green while the parameter is
    // still unused — delete this line as you write the real code.
    let _ = database_url;
    todo!(
        "let store = taskforge_storage::PostgresJobStore::connect(database_url).await?; \
         then return Ok(Arc::new(store))"
    )
}

/// ── Your step 2: assemble the pool, run it, drain it cleanly ────────────
///
/// `WorkerPool::run` takes a `tokio::sync::watch::Receiver<bool>`: every
/// worker loop keeps claiming jobs while the channel holds `false`, and
/// drains — finishes its current job, claims no more — once it flips to
/// `true`. `run` only returns when every in-flight job has completed.
/// So shutdown is a five-line dance:
///
/// ```rust,ignore
/// let mut pool = taskforge_worker::WorkerPool::new(store)
///     .with_concurrency(concurrency);
/// for handler in handlers {
///     pool = pool.register(handler);
/// }
/// let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
/// let running = tokio::spawn(pool.run(shutdown_rx));
/// shutdown.await;              // parks here until Ctrl-C / SIGTERM
/// shutdown_tx.send(true)?;     // tell every worker loop to drain
/// running.await?;              // returns once in-flight jobs finish
/// ```
async fn run_pool(
    store: Arc<dyn JobStore>,
    handlers: Vec<Arc<dyn JobHandler>>,
    concurrency: usize,
    shutdown: impl Future<Output = ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    // delete when you write the real code
    let _ = (store, handlers, concurrency, shutdown);
    todo!(
        "build the WorkerPool (new + with_concurrency + register each handler), \
         create the watch channel, tokio::spawn(pool.run(rx)), await `shutdown`, \
         send(true), then await the spawned task — the doc comment above shows every line"
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Provided: structured logs to stdout, filtered by RUST_LOG.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(
        concurrency = config.concurrency,
        "starting taskforge-worker"
    );

    let store = connect_store(&config.database_url).await?;
    let handlers: Vec<Arc<dyn JobHandler>> = vec![Arc::new(SendEmail)];
    run_pool(store, handlers, config.concurrency, shutdown_signal()).await?;

    tracing::info!("taskforge-worker stopped cleanly; all in-flight jobs finished");
    Ok(())
}
