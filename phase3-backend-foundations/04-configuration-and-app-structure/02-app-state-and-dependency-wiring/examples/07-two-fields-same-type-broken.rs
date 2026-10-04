//! DELIBERATELY BROKEN — expected: E0119
//!
//! `#[derive(FromRef)]` writes one `FromRef<AppState>` impl per field, keyed
//! by the field's *type*. Two fields of the same type means two impls for
//! that type, and `State<Arc<AtomicU32>>` could not tell them apart.
//!
//!     cargo build -p p3-04-02-app-state-and-dependency-wiring --example 07-two-fields-same-type-broken --features broken

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use axum::extract::{FromRef, State};
use axum::routing::get;
use axum::Router;

#[derive(Clone, FromRef)]
struct AppState {
    hits: Arc<AtomicU32>,
    misses: Arc<AtomicU32>,
}

async fn hits(State(hits): State<Arc<AtomicU32>>) -> String {
    hits.load(std::sync::atomic::Ordering::SeqCst).to_string()
}

fn main() {
    let state = AppState {
        hits: Arc::new(AtomicU32::new(0)),
        misses: Arc::new(AtomicU32::new(0)),
    };
    let _app: Router = Router::new().route("/hits", get(hits)).with_state(state);
}
