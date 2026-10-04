//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 04-fn-item-not-a-future-broken --features broken` and read the error.

use axum::Router;
use tokio::net::TcpListener;

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3161").await.unwrap();
    axum::serve(listener, Router::new())
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}
