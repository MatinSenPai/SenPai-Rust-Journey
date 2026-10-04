//! 3.4.3 — graceful shutdown, health and readiness.
//!
//! The ladder:
//!
//! - Implement: `Readiness::from_flags`, `Readiness::status`,
//!   `Readiness::body`, `shutdown_future`
//! - Build: `app`, `run`
//! - Challenge: `run_with_deadline`
//!
//! Everything under "provided" is already written. Run the tests with
//! `cargo test -p p3-04-03-graceful-shutdown-health-readiness`.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

/// The two facts `/ready` is computed from. Shared between the handlers and
/// the shutdown future, so it lives behind an `Arc`.
pub struct Lifecycle {
    draining: AtomicBool,
    dependency_up: AtomicBool,
}

impl Lifecycle {
    /// A fresh lifecycle: not draining, dependency up.
    pub fn new() -> Arc<Lifecycle> {
        Arc::new(Lifecycle {
            draining: AtomicBool::new(false),
            dependency_up: AtomicBool::new(true),
        })
    }

    pub fn start_draining(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    /// Tell the lifecycle whether the thing this service cannot work
    /// without (a database, from module 5 on) is reachable.
    pub fn set_dependency_up(&self, up: bool) {
        self.dependency_up.store(up, Ordering::SeqCst);
    }

    pub fn readiness(&self) -> Readiness {
        Readiness::from_flags(
            self.is_draining(),
            self.dependency_up.load(Ordering::SeqCst),
        )
    }
}

/// Whether this instance should be sent traffic right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    Ready,
    Draining,
    DependencyDown,
}

impl Readiness {
    /// Turns the two facts into a verdict.
    ///
    /// - `draining == true` gives `Draining`, whatever `dependency_up` says.
    /// - otherwise `dependency_up == false` gives `DependencyDown`.
    /// - otherwise `Ready`.
    pub fn from_flags(draining: bool, dependency_up: bool) -> Readiness {
        todo!("pick the verdict for these two flags; draining outranks a down dependency")
    }

    /// The status code `/ready` answers with: `200 OK` for `Ready`, and
    /// `503 Service Unavailable` for both `Draining` and `DependencyDown`.
    pub fn status(self) -> StatusCode {
        todo!("map each verdict to its status code")
    }

    /// The exact body text `/ready` answers with: `"ready"` for `Ready`,
    /// `"draining"` for `Draining`, `"dependency down"` for `DependencyDown`.
    pub fn body(self) -> &'static str {
        todo!("map each verdict to its body text")
    }
}

/// The future handed to `with_graceful_shutdown`. In order it:
///
/// 1. waits until `trigger` completes (until then it does nothing at all),
/// 2. calls `lifecycle.start_draining()` right away,
/// 3. waits `grace` more, and only then completes.
///
/// The `grace` wait is what gives a load balancer time to see the `503` on
/// `/ready`: the listener is still open during it.
pub async fn shutdown_future(
    trigger: impl Future<Output = ()>,
    lifecycle: Arc<Lifecycle>,
    grace: Duration,
) {
    todo!("wait for the trigger, mark the lifecycle as draining, then wait out the grace period")
}

/// Builds the router with two `GET` routes sharing `lifecycle` as state:
///
/// - `/health` (liveness): always `200` with the body `ok`, whatever the
///   lifecycle says.
/// - `/ready` (readiness): the status and body of `lifecycle.readiness()`.
pub fn app(lifecycle: Arc<Lifecycle>) -> Router {
    todo!("build the router with the liveness and readiness routes")
}

/// Serves `router` on `listener` until `shutdown_future(trigger, lifecycle,
/// grace)` completes, then returns `Ok(())` once every in-flight request has
/// finished. New connections must be refused from the moment the shutdown
/// future completes; requests already running must be allowed to finish.
pub async fn run(
    listener: TcpListener,
    router: Router,
    lifecycle: Arc<Lifecycle>,
    trigger: impl Future<Output = ()> + Send + 'static,
    grace: Duration,
) -> io::Result<()> {
    todo!("serve the router and stop gracefully when the shutdown future completes")
}

/// Challenge: like `run`, but once the shutdown future has completed and
/// `deadline` has passed with requests still running, give up and return
/// `Err` whose `kind()` is `io::ErrorKind::TimedOut`. The deadline starts
/// counting when the shutdown future completes, not when the server starts.
/// If the drain finishes within `deadline`, return `Ok(())` as `run` does.
pub async fn run_with_deadline(
    listener: TcpListener,
    router: Router,
    lifecycle: Arc<Lifecycle>,
    trigger: impl Future<Output = ()> + Send + 'static,
    grace: Duration,
    deadline: Duration,
) -> io::Result<()> {
    todo!("like run, but abandon the drain with a TimedOut error once the deadline passes")
}

// ---- provided, not exercises ----------------------------------------------

/// Resolves on Ctrl-C, or on SIGTERM on Unix. Windows has no SIGTERM, which
/// is why tests never use this and inject their own trigger instead.
pub async fn os_signal() {
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

/// Turns a `oneshot` receiver into a trigger that fires only when the sender
/// SENDS. A dropped sender never fires it (see example 06 for what happens
/// when it does).
pub async fn fired(rx: oneshot::Receiver<()>) {
    if rx.await.is_err() {
        std::future::pending::<()>().await;
    }
}

/// One `GET` over a fresh connection with `Connection: close`: returns the
/// status code and the body. A tiny stand-in for a real HTTP client.
pub async fn get_once(addr: SocketAddr, path: &str) -> io::Result<(u16, String)> {
    let mut stream = TcpStream::connect(addr).await?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).await?;
    let status = reply
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let body = reply.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    Ok((status, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draining_beats_a_down_dependency() {
        assert_eq!(Readiness::from_flags(false, true), Readiness::Ready);
        assert_eq!(
            Readiness::from_flags(false, false),
            Readiness::DependencyDown
        );
        assert_eq!(Readiness::from_flags(true, true), Readiness::Draining);
        assert_eq!(Readiness::from_flags(true, false), Readiness::Draining);
    }

    #[test]
    fn only_ready_is_200() {
        assert_eq!(Readiness::Ready.status(), StatusCode::OK);
        assert_eq!(
            Readiness::Draining.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            Readiness::DependencyDown.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn bodies_are_the_exact_words() {
        assert_eq!(Readiness::Ready.body(), "ready");
        assert_eq!(Readiness::Draining.body(), "draining");
        assert_eq!(Readiness::DependencyDown.body(), "dependency down");
    }

    #[tokio::test(start_paused = true)]
    async fn shutdown_future_marks_draining_then_waits_the_grace() {
        let lifecycle = Lifecycle::new();
        let (tx, rx) = oneshot::channel::<()>();
        let task = tokio::spawn(shutdown_future(
            fired(rx),
            lifecycle.clone(),
            Duration::from_secs(10),
        ));

        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(!lifecycle.is_draining(), "nothing has triggered it yet");

        tx.send(()).unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(lifecycle.is_draining(), "draining starts at the trigger");
        assert!(!task.is_finished(), "but the grace period is still running");

        tokio::time::sleep(Duration::from_secs(10)).await;
        assert!(task.is_finished(), "and it ends once the grace is over");
    }
}
