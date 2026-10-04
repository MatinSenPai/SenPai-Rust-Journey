//! `State<S>`: one value, attached once with `.with_state`, handed to every
//! request. Clones of the router share it; a fresh `AppState` does not.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 04-shared-state

use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::Request;
use axum::routing::post;
use axum::Router;
use tower::ServiceExt;

#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u64>>,
}

async fn visit(State(state): State<AppState>) -> String {
    let mut visits = state.visits.lock().unwrap();
    *visits += 1;
    format!("visit #{}", *visits)
}

fn app(state: AppState) -> Router {
    Router::new().route("/visit", post(visit)).with_state(state)
}

async fn send(label: &str, app: Router) {
    let request = Request::builder()
        .method("POST")
        .uri("/visit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("{label:<24} -> {}", String::from_utf8_lossy(&bytes));
}

#[tokio::main]
async fn main() {
    let shared = app(AppState::default());
    send("clone of the same router", shared.clone()).await;
    send("clone of the same router", shared.clone()).await;
    send("clone of the same router", shared).await;
    send("a brand-new AppState", app(AppState::default())).await;
}
