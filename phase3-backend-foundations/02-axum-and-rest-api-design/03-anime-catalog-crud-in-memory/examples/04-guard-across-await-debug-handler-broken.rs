//! DELIBERATELY BROKEN — expected: E0277
//!
//! Example 02 again, with `#[axum::debug_handler]` on the handler. The same
//! mistake (a `std::sync::MutexGuard` alive across an `.await`), but now
//! the compiler is asked to check the handler by itself, and says why.
//!
//!     cargo build -p p3-02-03-anime-catalog-crud-in-memory --example 04-guard-across-await-debug-handler-broken --features broken

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::get;
use axum::Router;

async fn audit_log() {}

#[axum::debug_handler]
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
