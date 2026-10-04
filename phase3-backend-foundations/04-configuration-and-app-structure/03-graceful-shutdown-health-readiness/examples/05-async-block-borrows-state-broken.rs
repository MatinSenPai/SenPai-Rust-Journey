//! DELIBERATELY BROKEN — expected: E0373
//! Run `cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 05-async-block-borrows-state-broken --features broken` and read the error.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let draining = Arc::new(AtomicBool::new(false));
    let listener = TcpListener::bind("127.0.0.1:3162").await.unwrap();
    axum::serve(listener, Router::new())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
            draining.store(true, Ordering::SeqCst);
        })
        .await
        .unwrap();
}
