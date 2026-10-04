//! DELIBERATELY BROKEN — expected: E0277
//!
//! `#[derive(Clone)]` is missing on the middleware service, and axum
//! clones services.
//!
//!     cargo build -p p3-02-04-tower-service-and-layer-middleware --example 08-service-not-clone-broken --features broken

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

struct Noop<S> {
    inner: S,
}

impl<S> Service<Request> for Noop<S>
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
        Box::pin(self.inner.call(request))
    }
}

fn main() {
    let _app: Router = Router::new()
        .route("/", get(|| async { "hello" }))
        .layer(NoopLayer);
}
