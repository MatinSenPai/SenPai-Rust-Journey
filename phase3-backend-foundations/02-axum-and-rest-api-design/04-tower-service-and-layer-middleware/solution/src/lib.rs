//! Solution for 3.2.4 — `tower::Service` and `Layer`: middleware by hand.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use tower::{Layer, Service};

// ------------------------------------------------------------------ LogLayer
//
// The lesson's worked example, complete. "The concept" walks through it and
// examples `03` and `04` use it.

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

/// A [`Layer`] that wraps a service in [`ResponseTime`].
#[derive(Clone)]
pub struct ResponseTimeLayer;

impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTime<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseTime { inner }
    }
}

/// Middleware that times each response and reports it in a header.
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
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = future.await?;
            let elapsed_ms = start.elapsed().as_millis() as u64;
            response
                .headers_mut()
                .insert("x-response-time-ms", HeaderValue::from(elapsed_ms));
            Ok(response)
        })
    }
}

// ------------------------------------------------------- Build: maintenance

/// A [`Layer`] that wraps a service in [`Maintenance`]. Every service it
/// wraps shares the same `flag`.
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

    fn layer(&self, inner: S) -> Self::Service {
        Maintenance {
            inner,
            flag: Arc::clone(&self.flag),
        }
    }
}

/// Middleware that answers `503` by itself while the shared flag is on.
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
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        if self.flag.load(Ordering::SeqCst) {
            let response = Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("down for maintenance"))
                .unwrap();
            return Box::pin(async move { Ok(response) });
        }
        Box::pin(self.inner.call(request))
    }
}
