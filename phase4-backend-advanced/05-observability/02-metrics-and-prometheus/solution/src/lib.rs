//! A tiny axum app instrumented with the `metrics` facade + Prometheus
//! exporter. See `README.md` for the theory (logs vs. metrics) this builds
//! on. Reuses the exact `OnceLock<PrometheusHandle>` pattern from
//! `capstone-taskforge/taskforge-api` — read the doc comment on
//! `metrics_handle` below before touching anything else.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct Widget {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWidgetRequest {
    pub name: String,
}

#[derive(Clone)]
struct AppState {
    metrics_handle: PrometheusHandle,
    widgets: Arc<Mutex<HashMap<u64, Widget>>>,
    next_id: Arc<AtomicU64>,
}

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// `PrometheusBuilder::install_recorder` installs a *global* recorder —
/// calling it twice in the same process panics. `build_router` may
/// legitimately be called more than once per process (every test in this
/// lesson does), so the install happens exactly once, here, and every
/// caller after the first just gets a clone of the same handle. Copied
/// verbatim from `capstone-taskforge/taskforge-api/src/lib.rs` — this trap
/// has already been hit once in this repo, no need to hit it again.
fn metrics_handle() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("installing the Prometheus recorder should only fail if called twice")
        })
        .clone()
}

/// Creates a widget: `POST /widgets`. Should increment a
/// `widgets_created_total` counter and record how long the handler took in
/// a `widget_create_duration_seconds` histogram — the two metric *kinds*
/// this lesson is about (see README.md): a counter for "how many," a
/// histogram for "how long, distributed."
async fn create_widget(
    State(state): State<AppState>,
    Json(req): Json<CreateWidgetRequest>,
) -> (StatusCode, Json<Widget>) {
    let start = Instant::now();

    let id = state.next_id.fetch_add(1, Ordering::SeqCst);
    let widget = Widget { id, name: req.name };
    state.widgets.lock().unwrap().insert(id, widget.clone());

    metrics::counter!("widgets_created_total").increment(1);
    metrics::histogram!("widget_create_duration_seconds").record(start.elapsed().as_secs_f64());

    (StatusCode::CREATED, Json(widget))
}

/// Looks up a widget: `GET /widgets/<id>`. Should increment
/// `widget_lookups_total` on every call — with a `result` label so the two
/// outcomes (`hit` vs `miss`) are queryable separately in Prometheus — and
/// record the same handler-duration histogram idea as `create_widget`.
async fn get_widget(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Widget>, StatusCode> {
    let start = Instant::now();

    let found = state.widgets.lock().unwrap().get(&id).cloned();

    metrics::histogram!("widget_lookup_duration_seconds").record(start.elapsed().as_secs_f64());

    match found {
        Some(widget) => {
            metrics::counter!("widget_lookups_total", "result" => "hit").increment(1);
            Ok(Json(widget))
        }
        None => {
            metrics::counter!("widget_lookups_total", "result" => "miss").increment(1);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Renders the current state of every registered metric in the Prometheus
/// text exposition format — this is the literal HTTP response a real
/// Prometheus server's scrape job receives when it polls `/metrics`.
async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

/// Wires the three routes together.
pub fn build_router() -> Router {
    let state = AppState {
        metrics_handle: metrics_handle(),
        widgets: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(AtomicU64::new(1)),
    };

    Router::new()
        .route("/widgets", post(create_widget))
        .route("/widgets/{id}", get(get_widget))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}
