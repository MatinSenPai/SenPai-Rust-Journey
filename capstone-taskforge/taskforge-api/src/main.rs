//! Service binary for `taskforge-api` — **this file is your exercise**.
//!
//! The library half of this crate (`src/lib.rs` and friends) is finished,
//! reviewed reference code: it assembles every route, the bearer-token
//! middleware, the Prometheus recorder, and the OpenAPI document, and all
//! of it is covered by `tests/api_test.rs`. What it deliberately does NOT
//! do is *run* — something has to read config, connect to Postgres, and
//! actually serve HTTP. That something is this binary, and wiring it is
//! your job.
//!
//! Provided for you (read it — it's part of the lesson):
//! - tracing/logging setup in `main`, filtered by `RUST_LOG`
//! - [`Config::from_env`] — fail-fast environment config
//! - [`shutdown_signal`] — the graceful-shutdown future (see its docs;
//!   graceful shutdown is the one piece of the stack no earlier lesson
//!   covered, so it's given to you whole)
//!
//! You fill in the three `todo!()`s: [`connect_store`], [`build_app`],
//! and [`serve`]. Each one's doc comment is a near-answer hint. When all
//! three are done:
//!
//! ```sh
//! docker compose -f ../docker-compose.yml up -d postgres
//! DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
//!   API_TOKEN=dev-token cargo run -p taskforge-api
//! curl http://localhost:8080/health          # -> ok
//! ```
//!
//! …and then `docker compose up --build` should bring up the whole stack
//! (see `../docker-compose.yml` and the capstone README's "What you
//! build" section).

use std::future::Future;
use std::sync::Arc;
use taskforge_core::JobStore;

/// Everything this service reads from the environment, gathered in one
/// place and validated once at startup. A service that's missing config
/// should fail *now*, loudly — not on the first request at 3am.
struct Config {
    /// e.g. `postgres://taskforge:taskforge@localhost:5432/taskforge`
    database_url: String,
    /// Where to listen. Defaults to `0.0.0.0:8080` (all interfaces — the
    /// right default *inside a container*, where the container boundary
    /// is the firewall).
    bind_addr: String,
    /// The shared bearer token gating every `/jobs*` route.
    api_token: String,
}

impl Config {
    fn from_env() -> Result<Config, String> {
        let require = |name: &str| {
            std::env::var(name).map_err(|_| format!("required env var {name} is not set"))
        };
        Ok(Config {
            database_url: require("DATABASE_URL")?,
            api_token: require("API_TOKEN")?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        })
    }
}

/// Resolves when the process is asked to stop: Ctrl-C (SIGINT) anywhere,
/// plus SIGTERM on Unix — SIGTERM is what `docker stop`, Kubernetes, and
/// systemd actually send. A service that only listens for Ctrl-C looks
/// fine on a laptop and then gets SIGKILLed mid-request in production
/// after the 10-second grace period expires.
///
/// You don't call `.await` on this directly in a request loop — you hand
/// the future to axum: `.with_graceful_shutdown(shutdown_signal())`.
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
    tracing::info!("shutdown signal received; draining in-flight requests");
}

/// ── Your step 1: Postgres pool + migrations + store ─────────────────────
///
/// `taskforge_storage::PostgresJobStore::connect(database_url)` already
/// does all three things this step needs (read its source — it's short):
///   1. builds a `PgPool` capped at 10 connections,
///   2. runs everything in `taskforge-storage/migrations/` via
///      `sqlx::migrate!` (idempotent: already-applied migrations are
///      skipped, and concurrent services racing to migrate take an
///      advisory lock, so api and worker can both call this at startup),
///   3. wraps the pool in the store.
///
/// Your job: call it, `.await?` it, and hand the result back as the trait
/// object the rest of the system depends on.
async fn connect_store(
    database_url: &str,
) -> Result<Arc<dyn JobStore>, Box<dyn std::error::Error>> {
    // Keeps `cargo clippy -- -D warnings` green while the parameter is
    // still unused — delete this line as you write the real code.
    let _ = database_url;
    todo!(
        "let store = taskforge_storage::PostgresJobStore::connect(database_url).await?; \
         then return Ok(Arc::new(store)) — the Arc<dyn JobStore> coercion is automatic"
    )
}

/// ── Your step 2: build the application ──────────────────────────────────
///
/// The library half already assembles every route, the auth middleware,
/// `/metrics`, and `/api-docs/openapi.json`. This is one function call.
fn build_app(store: Arc<dyn JobStore>, api_token: String) -> axum::Router {
    let _ = (store, api_token); // delete when you write the real code
    todo!("one call: taskforge_api::build_router(store, api_token)")
}

/// ── Your step 3: bind, serve, shut down gracefully ──────────────────────
///
/// The pattern to reproduce — this is THE production axum idiom:
///
/// ```rust,ignore
/// let listener = tokio::net::TcpListener::bind(bind_addr).await?;
/// tracing::info!("listening on {}", listener.local_addr()?);
/// axum::serve(listener, app)
///     .with_graceful_shutdown(shutdown)
///     .await?;
/// ```
///
/// Without `.with_graceful_shutdown(...)`, a Ctrl-C or `docker stop`
/// kills the process mid-request and in-flight responses are dropped on
/// the floor. With it, axum stops accepting *new* connections the moment
/// `shutdown` resolves, finishes every in-flight request, and only then
/// returns — which is why `main` can log "stopped cleanly" afterwards.
async fn serve(
    app: axum::Router,
    bind_addr: &str,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = (app, bind_addr, shutdown); // delete when you write the real code
    todo!(
        "bind a tokio::net::TcpListener at bind_addr, then \
         axum::serve(listener, app).with_graceful_shutdown(shutdown).await? — \
         the doc comment above shows the exact shape"
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Provided: structured logs to stdout. RUST_LOG picks the filter
    // (e.g. RUST_LOG=taskforge_api=debug,info); defaults to "info".
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(bind_addr = %config.bind_addr, "starting taskforge-api");

    let store = connect_store(&config.database_url).await?;
    let app = build_app(store, config.api_token);
    serve(app, &config.bind_addr, shutdown_signal()).await?;

    tracing::info!("taskforge-api stopped cleanly");
    Ok(())
}
