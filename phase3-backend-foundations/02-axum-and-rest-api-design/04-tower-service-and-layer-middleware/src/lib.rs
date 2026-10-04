//! Exercises for 3.2.4 — `tower::Service` and `Layer`: middleware by hand.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::task::{Context, Poll};

// Imports you will probably want for the exercises below.
#[allow(unused_imports)]
use axum::body::Body;
use axum::extract::Request;
#[allow(unused_imports)]
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
#[allow(unused_imports)]
use std::sync::atomic::Ordering;
#[allow(unused_imports)]
use std::time::Instant;
use tower::{Layer, Service};

// ------------------------------------------------------------------ LogLayer
//
// The lesson's worked example, complete and given. "The concept" walks through
// it and examples `03` and `04` use it. Read it before you write your own.

/// A [`Layer`] that wraps a service in [`Log`]. `name` is printed on every
/// line, so two of these in one app can be told apart.
#[derive(Clone)]
pub struct LogLayer {
    name: &'static str,
}

impl LogLayer {
    pub fn new(name: &'static str) -> Self {
        LogLayer { name }
    }
}

impl<S> Layer<S> for LogLayer {
    type Service = Log<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Log {
            inner,
            name: self.name,
        }
    }
}

/// Prints one line when a request goes in and one when its response comes
/// out, and changes nothing else.
#[derive(Clone)]
pub struct Log<S> {
    inner: S,
    name: &'static str,
}

impl<S> Service<Request> for Log<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let name = self.name;
        println!(
            "{name}: request in  ({} {})",
            request.method(),
            request.uri()
        );
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;
            println!("{name}: response out ({})", response.status());
            Ok(response)
        })
    }
}

// ---------------------------------------------------------- Implement: timing

/// A [`Layer`] that wraps a service in [`ResponseTime`]. It has no
/// configuration, so it is a unit struct.
#[derive(Clone)]
pub struct ResponseTimeLayer;

impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTime<S>;

    /// Returns `inner` wrapped in a [`ResponseTime`].
    fn layer(&self, inner: S) -> Self::Service {
        todo!("wrap `inner` in a ResponseTime")
    }
}

/// Middleware that times each response and reports it in a header.
///
/// For every request it forwards the request to `inner` unchanged, waits for
/// the response, and adds one header to it: `x-response-time-ms`, whose value
/// is the whole milliseconds that passed between `call` being invoked and
/// the inner response arriving, written as plain decimal digits (`"0"`,
/// `"7"`, `"1532"`). If the response already carries an
/// `x-response-time-ms` header, it is replaced, not duplicated. Status code
/// and body are left alone. If `inner` fails with an error, the error is
/// passed on unchanged and no header is added.
///
/// `poll_ready` must report exactly what `inner.poll_ready` reports.
#[derive(Clone)]
pub struct ResponseTime<S> {
    inner: S,
}

impl<S> Service<Request> for ResponseTime<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        todo!("report whether the inner service is ready")
    }

    fn call(&mut self, request: Request) -> Self::Future {
        todo!("forward the request and stamp the elapsed milliseconds on the response")
    }
}

// ------------------------------------------------------- Build: maintenance

/// A [`Layer`] that wraps a service in [`Maintenance`]. Every service it
/// wraps shares the same `flag`: the layer keeps one `Arc` and gives each
/// service its own clone of that `Arc`, never a copy of the bool.
#[derive(Clone)]
pub struct MaintenanceLayer {
    flag: Arc<AtomicBool>,
}

impl MaintenanceLayer {
    pub fn new(flag: Arc<AtomicBool>) -> Self {
        MaintenanceLayer { flag }
    }
}

impl<S> Layer<S> for MaintenanceLayer {
    type Service = Maintenance<S>;

    /// Returns `inner` wrapped in a [`Maintenance`] that shares this layer's flag.
    fn layer(&self, inner: S) -> Self::Service {
        todo!("wrap `inner` in a Maintenance that shares this layer's flag")
    }
}

/// Middleware that answers `503` by itself while the shared flag is on.
///
/// On every call it reads the flag (with `Ordering::SeqCst`) at that moment.
/// Flag on: it does NOT call `inner` at all, and returns a response with
/// status `503 Service Unavailable` and the body `down for maintenance`
/// (exactly those bytes, no trailing newline). Flag off: the request goes
/// to `inner` and its response comes back unchanged. Flipping the flag
/// takes effect on the very next request, with no rebuild of the app.
///
/// `poll_ready` must report exactly what `inner.poll_ready` reports.
#[derive(Clone)]
pub struct Maintenance<S> {
    inner: S,
    flag: Arc<AtomicBool>,
}

impl<S> Service<Request> for Maintenance<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        todo!("report whether the inner service is ready")
    }

    fn call(&mut self, request: Request) -> Self::Future {
        todo!("answer 503 by itself while the flag is on, otherwise forward the request")
    }
}
