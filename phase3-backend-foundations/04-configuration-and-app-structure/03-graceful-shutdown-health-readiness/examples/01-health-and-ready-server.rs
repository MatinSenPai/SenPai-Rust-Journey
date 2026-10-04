//! A real server on a fixed port with the two endpoints a deployment checks,
//! and a graceful shutdown on Ctrl-C (and on SIGTERM on Unix).
//!
//!     cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 01-health-and-ready-server
//!
//! Then, from a second terminal:  curl -i http://127.0.0.1:3160/health

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

async fn ready(State(draining): State<Arc<AtomicBool>>) -> (StatusCode, &'static str) {
    if draining.load(Ordering::SeqCst) {
        (StatusCode::SERVICE_UNAVAILABLE, "draining")
    } else {
        (StatusCode::OK, "ready")
    }
}

async fn os_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

#[tokio::main]
async fn main() {
    let draining = Arc::new(AtomicBool::new(false));
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .with_state(draining.clone());

    let listener = TcpListener::bind("127.0.0.1:3160").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            os_signal().await;
            draining.store(true, Ordering::SeqCst);
            println!("signal received: no new connections, finishing in-flight requests");
        })
        .await
        .unwrap();
    println!("all requests finished, exiting");
}
