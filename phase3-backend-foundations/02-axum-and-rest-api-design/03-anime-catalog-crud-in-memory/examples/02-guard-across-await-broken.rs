//! DELIBERATELY BROKEN — expected: E0277
//!
//! A handler that keeps a `std::sync::MutexGuard` alive across an `.await`.
//! The guard is not `Send`, so the handler's future is not `Send`, and axum
//! refuses to accept it as a handler.
//!
//!     cargo build -p p3-02-03-anime-catalog-crud-in-memory --example 02-guard-across-await-broken --features broken

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::get;
use axum::Router;

async fn audit_log() {}

async fn count(State(counter): State<Arc<Mutex<u32>>>) -> String {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    audit_log().await;
    format!("{}", *guard)
}

fn main() {
    let _app: Router = Router::new()
        .route("/count", get(count))
        .with_state(Arc::new(Mutex::new(0)));
}
