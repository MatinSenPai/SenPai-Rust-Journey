//! The challenge: a drain that can be given up on.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};
use tokio::time::timeout;

use p3_04_03_graceful_shutdown_health_readiness::{
    app, fired, get_once, run_with_deadline, Lifecycle,
};

#[tokio::test]
async fn a_request_that_never_finishes_cannot_hold_shutdown_forever() {
    let lifecycle = Lifecycle::new();
    let started = Arc::new(Notify::new());
    let started_h = started.clone();
    let router = app(lifecycle.clone()).route(
        "/stuck",
        get(move || async move {
            started_h.notify_one();
            std::future::pending::<&'static str>().await
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, rx) = oneshot::channel::<()>();
    let server = tokio::spawn(run_with_deadline(
        listener,
        router,
        lifecycle,
        fired(rx),
        Duration::ZERO,
        Duration::from_millis(100),
    ));

    tokio::spawn(get_once(addr, "/stuck"));
    timeout(Duration::from_secs(10), started.notified())
        .await
        .unwrap();
    stop.send(()).unwrap();

    let result = timeout(Duration::from_secs(10), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
}

#[tokio::test]
async fn a_drain_that_finishes_in_time_is_ok() {
    let lifecycle = Lifecycle::new();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (stop, rx) = oneshot::channel::<()>();
    let server = tokio::spawn(run_with_deadline(
        listener,
        app(lifecycle.clone()),
        lifecycle,
        fired(rx),
        Duration::ZERO,
        Duration::from_secs(30),
    ));
    stop.send(()).unwrap();
    timeout(Duration::from_secs(10), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
