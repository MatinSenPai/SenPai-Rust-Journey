//! Real servers on ephemeral ports (`127.0.0.1:0`), driven over plain TCP.
//! No sleeps: every wait is "until this observable thing happens", with a
//! generous timeout that only matters when the test is already failing.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use p3_04_03_graceful_shutdown_health_readiness_solution::{app, fired, get_once, run, Lifecycle};

const PATIENCE: Duration = Duration::from_secs(10);

struct Server {
    addr: SocketAddr,
    lifecycle: Arc<Lifecycle>,
    stop: oneshot::Sender<()>,
    /// The `/slow` handler tells the test it has started...
    started: Arc<Notify>,
    /// ...and then waits for the test to let it finish.
    release: Arc<Notify>,
    join: JoinHandle<io::Result<()>>,
}

async fn start(grace: Duration) -> Server {
    let lifecycle = Lifecycle::new();
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let (started_h, release_h) = (started.clone(), release.clone());
    let router = app(lifecycle.clone()).route(
        "/slow",
        get(move || async move {
            started_h.notify_one();
            release_h.notified().await;
            "slow done"
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, rx) = oneshot::channel::<()>();
    let join = tokio::spawn(run(listener, router, lifecycle.clone(), fired(rx), grace));
    Server {
        addr,
        lifecycle,
        stop,
        started,
        release,
        join,
    }
}

/// Polls until a new connection is refused, i.e. the listener is gone.
async fn wait_until_refused(addr: SocketAddr) {
    timeout(PATIENCE, async {
        while TcpStream::connect(addr).await.is_ok() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the listener should have been dropped");
}

#[tokio::test]
async fn health_and_ready_answer_before_any_shutdown() {
    let server = start(Duration::ZERO).await;
    assert_eq!(
        get_once(server.addr, "/health").await.unwrap(),
        (200, "ok".into())
    );
    assert_eq!(
        get_once(server.addr, "/ready").await.unwrap(),
        (200, "ready".into())
    );
}

#[tokio::test]
async fn in_flight_request_finishes_and_new_connections_are_refused() {
    let server = start(Duration::ZERO).await;
    let slow = tokio::spawn(get_once(server.addr, "/slow"));
    timeout(PATIENCE, server.started.notified()).await.unwrap();

    server.stop.send(()).unwrap();
    wait_until_refused(server.addr).await;
    assert!(
        !server.join.is_finished(),
        "serve must wait for the request that is still running"
    );

    server.release.notify_one();
    let (status, body) = slow.await.unwrap().unwrap();
    assert_eq!((status, body.as_str()), (200, "slow done"));
    timeout(PATIENCE, server.join)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn ready_turns_503_while_draining_but_health_stays_200() {
    // A long grace keeps the listener open so we can still ask. The test
    // never waits for it: the runtime drops the server task at the end.
    let server = start(Duration::from_secs(60)).await;
    server.stop.send(()).unwrap();

    let (status, body) = timeout(PATIENCE, async {
        loop {
            let reply = get_once(server.addr, "/ready").await.unwrap();
            if reply.0 == 503 {
                break reply;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!((status, body.as_str()), (503, "draining"));
    assert!(server.lifecycle.is_draining());
    assert_eq!(
        get_once(server.addr, "/health").await.unwrap(),
        (200, "ok".into())
    );
}

#[tokio::test]
async fn a_down_dependency_fails_ready_but_not_health() {
    let server = start(Duration::ZERO).await;
    server.lifecycle.set_dependency_up(false);
    assert_eq!(
        get_once(server.addr, "/ready").await.unwrap(),
        (503, "dependency down".into())
    );
    assert_eq!(
        get_once(server.addr, "/health").await.unwrap(),
        (200, "ok".into())
    );

    server.lifecycle.set_dependency_up(true);
    assert_eq!(
        get_once(server.addr, "/ready").await.unwrap(),
        (200, "ready".into())
    );
}

#[tokio::test]
async fn an_idle_keep_alive_connection_is_closed_by_shutdown() {
    let server = start(Duration::ZERO).await;
    let mut stream = TcpStream::connect(server.addr).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: test\r\n\r\n")
        .await
        .unwrap();
    let mut reply = String::new();
    let mut buf = [0u8; 256];
    while !reply.ends_with("ok") {
        let n = timeout(PATIENCE, stream.read(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert!(n > 0, "the server closed before answering");
        reply.push_str(&String::from_utf8_lossy(&buf[..n]));
    }

    server.stop.send(()).unwrap();
    let n = timeout(PATIENCE, stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(n, 0, "end of stream: the idle connection was closed");
    timeout(PATIENCE, server.join)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
