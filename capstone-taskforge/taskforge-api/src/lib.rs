//! The REST surface: enqueue/inspect/list/cancel jobs, bearer-token
//! auth-gated, plus a `/metrics` Prometheus endpoint and an unauthenticated
//! `/health` check. Depends only on `Arc<dyn JobStore>` (from
//! `taskforge-core`), never on a concrete storage implementation — see
//! `../docs/adr/0001-architecture-overview.md`. This is what makes every
//! test in `tests/` run against `InMemoryJobStore` with zero infrastructure.

mod auth;
mod error;
mod handlers;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::{Arc, OnceLock};
use taskforge_core::JobStore;

pub use error::{ApiError, ErrorBody};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn JobStore>,
    pub auth_token: String,
    pub metrics_handle: PrometheusHandle,
}

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// `PrometheusBuilder::install_recorder` installs a *global* recorder —
/// calling it twice in the same process panics. `build_router` may
/// legitimately be called more than once per process (every test in
/// `tests/` does), so the install happens exactly once, here, and every
/// caller after the first just gets a clone of the same handle.
fn metrics_handle() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("installing the Prometheus recorder should only fail if called twice")
        })
        .clone()
}

/// Builds the full router. `auth_token` gates every `/jobs*` route;
/// `/health` and `/metrics` stay open (a real deployment would put
/// `/metrics` behind network-level access control instead — scraping it
/// shouldn't need the same bearer token every other client uses).
pub fn build_router(store: Arc<dyn JobStore>, auth_token: String) -> Router {
    let metrics_handle = metrics_handle();

    let state = AppState {
        store,
        auth_token,
        metrics_handle,
    };

    let jobs_routes = Router::new()
        .route(
            "/jobs",
            post(handlers::enqueue_job).get(handlers::list_jobs),
        )
        .route("/jobs/{id}", get(handlers::get_job))
        .route("/jobs/{id}/cancel", post(handlers::cancel_job))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_bearer_token,
        ));

    Router::new()
        .route("/health", get(handlers::health))
        .route("/metrics", get(handlers::metrics_handler))
        .merge(jobs_routes)
        .with_state(state)
}
