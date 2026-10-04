//! DELIBERATELY BROKEN — expected: E0277
//!
//! The state struct does not implement `Clone`, so `axum` cannot hand a copy
//! to each request.
//!
//!     cargo build -p p3-04-02-app-state-and-dependency-wiring --example 04-state-not-clone-broken --features broken

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::get;
use axum::Router;

struct AppState {
    visits: Arc<Mutex<u64>>,
}

async fn visits(State(state): State<AppState>) -> String {
    format!("{} visits", state.visits.lock().unwrap())
}

fn main() {
    let state = AppState {
        visits: Arc::new(Mutex::new(0)),
    };
    let _app: Router = Router::new()
        .route("/visits", get(visits))
        .with_state(state);
}
