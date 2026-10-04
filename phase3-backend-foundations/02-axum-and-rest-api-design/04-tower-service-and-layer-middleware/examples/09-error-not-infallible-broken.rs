//! DELIBERATELY BROKEN — expected: E0277
//!
//! The middleware's error type is `String`, not `Infallible`, and axum's
//! `.layer` requires every service error to convert into `Infallible`.
//!
//!     cargo build -p p3-02-04-tower-service-and-layer-middleware --example 09-error-not-infallible-broken --features broken

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::{Layer, Service};

#[derive(Clone)]
struct NoopLayer;

impl<S> Layer<S> for NoopLayer {
    type Service = Noop<S>;
    fn layer(&self, inner: S) -> Noop<S> {
        Noop { inner }
    }
}

#[derive(Clone)]
struct Noop<S> {
    inner: S,
}

impl<S> Service<Request> for Noop<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Response, String>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), String>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move { future.await.map_err(|_| "inner failed".to_string()) })
    }
}

fn main() {
    let _app: Router = Router::new()
        .route("/", get(|| async { "hello" }))
        .layer(NoopLayer);
}
