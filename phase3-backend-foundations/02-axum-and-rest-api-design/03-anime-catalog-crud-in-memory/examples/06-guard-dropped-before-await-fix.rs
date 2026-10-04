//! The fix for examples 02 and 04: finish with the lock in a small block, so
//! the guard is gone before the `.await`. Same handler, now `Send`.
//!
//!     cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 06-guard-dropped-before-await-fix

use std::sync::{Arc, Mutex};

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

async fn audit_log() {}

#[axum::debug_handler]
async fn count(State(counter): State<Arc<Mutex<u32>>>) -> String {
    let now = {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
        *guard
    }; // the guard is dropped here, before any `.await`
    audit_log().await;
    format!("{now}")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/count", get(count))
        .with_state(Arc::new(Mutex::new(0)));
    for _ in 0..2 {
        let request = Request::get("/count").body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("GET /count -> {}", String::from_utf8_lossy(&body));
    }
}
