//! DELIBERATELY BROKEN — expected: E0277
//!
//! `Clock` does not say `Send + Sync`, so `Arc<dyn Clock>` cannot be shared
//! between the threads that run handlers.
//!
//!     cargo build -p p3-04-02-app-state-and-dependency-wiring --example 05-dyn-without-send-sync-broken --features broken

use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::Router;

trait Clock {
    fn now(&self) -> u64;
}

#[derive(Clone)]
struct AppState {
    clock: Arc<dyn Clock>,
}

async fn now(State(state): State<AppState>) -> String {
    state.clock.now().to_string()
}

struct Fixed;

impl Clock for Fixed {
    fn now(&self) -> u64 {
        0
    }
}

fn main() {
    let state = AppState {
        clock: Arc::new(Fixed),
    };
    let _app: Router = Router::new().route("/now", get(now)).with_state(state);
}
