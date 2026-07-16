//! Service binary for `taskforge-scheduler` — **this file is your
//! exercise**, and the smallest of the three (do it last; by now the
//! shape should feel routine — that's the point).
//!
//! The library half (`Scheduler` in `src/lib.rs`) is finished reference
//! code. This binary reads config, connects to Postgres, registers a
//! demo schedule, and runs the scheduler until shutdown.
//!
//! Provided: tracing setup, [`Config::from_env`], the demo schedule in
//! `main`, and [`shutdown_signal`]. You fill in [`connect_store`] and
//! [`run_scheduler`]. When both are done:
//!
//! ```sh
//! docker compose -f ../docker-compose.yml up -d postgres
//! DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
//!   cargo run -p taskforge-scheduler
//! ```
//!
//! …and a `send_email` job appears in the queue every 30 seconds — which
//! the worker binary (once running) picks up and processes.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use taskforge_core::JobStore;
use taskforge_scheduler::ScheduledJob;

/// The scheduler only needs to reach the database — it never serves
/// traffic and never runs job handlers.
struct Config {
    database_url: String,
}

impl Config {
    fn from_env() -> Result<Config, String> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "required env var DATABASE_URL is not set".to_string())?;
        Ok(Config { database_url })
    }
}

/// Resolves on Ctrl-C (SIGINT), plus SIGTERM on Unix — what `docker stop`
/// sends. See the longer discussion in `taskforge-api/src/main.rs`.
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
    tracing::info!("shutdown signal received");
}

/// ── Your step 1: Postgres pool + migrations + store ─────────────────────
///
/// Same as in the other two binaries:
/// `taskforge_storage::PostgresJobStore::connect(database_url)` builds
/// the pool, runs migrations (idempotent, advisory-locked), and wraps it.
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

/// ── Your step 2: run the scheduler until shutdown ───────────────────────
///
/// Same watch-channel dance as the worker binary, minus handlers:
///
/// ```rust,ignore
/// let mut scheduler = taskforge_scheduler::Scheduler::new(store);
/// for schedule in schedules {
///     scheduler = scheduler.with_schedule(schedule);
/// }
/// let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
/// let running = tokio::spawn(scheduler.run(shutdown_rx));
/// shutdown.await;
/// shutdown_tx.send(true)?;
/// running.await?;
/// ```
async fn run_scheduler(
    store: Arc<dyn JobStore>,
    schedules: Vec<ScheduledJob>,
    shutdown: impl Future<Output = ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = (store, schedules, shutdown); // delete when you write the real code
    todo!(
        "build the Scheduler (new + with_schedule for each entry), create the \
         watch channel, tokio::spawn(scheduler.run(rx)), await `shutdown`, \
         send(true), then await the spawned task"
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
    tracing::info!("starting taskforge-scheduler");

    let store = connect_store(&config.database_url).await?;

    // Provided: a demo recurring job. `send_email` is deliberately the
    // job_type the worker binary registers a handler for, so the full
    // stack visibly processes scheduled work with zero extra setup.
    let schedules = vec![ScheduledJob::new(
        "send_email",
        serde_json::json!({"to": "digest@example.com", "reason": "scheduled digest"}),
        Duration::from_secs(30),
    )];

    run_scheduler(store, schedules, shutdown_signal()).await?;

    tracing::info!("taskforge-scheduler stopped cleanly");
    Ok(())
}
