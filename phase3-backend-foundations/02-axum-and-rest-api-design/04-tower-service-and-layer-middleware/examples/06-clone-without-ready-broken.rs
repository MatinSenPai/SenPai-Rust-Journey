//! DELIBERATELY BROKEN — expected: a run-time panic, "poll_ready must be called first"
//!
//! The middleware clones its inner service inside `call` and calls the
//! clone. The service that said "ready" is not the one that gets called.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 06-clone-without-ready-broken --features broken

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tower::limit::ConcurrencyLimit;
use tower::{service_fn, Service, ServiceExt};

#[derive(Clone)]
struct Wrapper<S> {
    inner: S,
}

impl<S> Service<u32> for Wrapper<S>
where
    S: Service<u32> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: u32) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(request).await })
    }
}

#[tokio::main]
async fn main() {
    let echo = service_fn(|n: u32| async move { Ok::<_, Infallible>(n) });
    let mut svc = Wrapper {
        inner: ConcurrencyLimit::new(echo, 1),
    };
    let answer = svc.ready().await.unwrap().call(7).await.unwrap();
    println!("answer = {answer}");
}
