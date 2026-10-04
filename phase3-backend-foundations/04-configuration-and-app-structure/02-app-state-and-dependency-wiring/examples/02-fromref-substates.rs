//! `#[derive(FromRef)]` lets a handler ask for one field of the state
//! instead of the whole struct. Each handler below names only what it uses.
//!
//!     cargo run -p p3-04-02-app-state-and-dependency-wiring --example 02-fromref-substates

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::{FromRef, State};
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

#[derive(Clone)]
struct Greeting(&'static str);

#[derive(Clone, FromRef)]
struct AppState {
    greeting: Greeting,
    hits: Arc<AtomicU32>,
}

async fn hello(State(greeting): State<Greeting>) -> String {
    greeting.0.to_string()
}

async fn count(State(hits): State<Arc<AtomicU32>>) -> String {
    format!("hit number {}", hits.fetch_add(1, Ordering::SeqCst) + 1)
}

async fn both(State(greeting): State<Greeting>, State(hits): State<Arc<AtomicU32>>) -> String {
    format!(
        "{} (hits so far: {})",
        greeting.0,
        hits.load(Ordering::SeqCst)
    )
}

#[tokio::main]
async fn main() {
    let state = AppState {
        greeting: Greeting("konnichiwa"),
        hits: Arc::new(AtomicU32::new(0)),
    };
    let app: Router = Router::new()
        .route("/hello", get(hello))
        .route("/count", get(count))
        .route("/both", get(both))
        .with_state(state);

    for uri in ["/hello", "/count", "/count", "/both"] {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("GET {uri} -> {}", String::from_utf8_lossy(&body));
    }
}
