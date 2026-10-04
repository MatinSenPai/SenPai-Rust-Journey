//! DELIBERATELY BROKEN — expected: E0308
//!
//! A handler asks for `State<AppState>`, but the router is never given one:
//! the `.with_state(...)` call is missing.
//!
//!     cargo build -p p3-02-01-routing-handlers-extractors --example 08-missing-with-state-broken --features broken

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::get;
use axum::Router;

#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u64>>,
}

async fn visits(State(state): State<AppState>) -> String {
    format!("{} visits", *state.visits.lock().unwrap())
}

fn app() -> Router {
    Router::new().route("/visits", get(visits))
}

fn main() {
    let _app = app();
}
