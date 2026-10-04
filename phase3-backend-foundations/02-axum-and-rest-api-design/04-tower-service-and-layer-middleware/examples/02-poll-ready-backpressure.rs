//! `poll_ready` is backpressure: a service that allows one request at a time
//! says "not ready" while one is in flight.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 02-poll-ready-backpressure

use std::convert::Infallible;
use std::time::Duration;

use tower::limit::ConcurrencyLimit;
use tower::{service_fn, Service, ServiceExt};

#[tokio::main]
async fn main() {
    let echo = service_fn(|n: u32| async move { Ok::<_, Infallible>(n) });
    let mut svc = ConcurrencyLimit::new(echo, 1);

    let first = svc.ready().await.unwrap().call(1);
    println!("first request is in flight (its future is not awaited yet)");

    let second = tokio::time::timeout(Duration::from_millis(50), svc.ready()).await;
    let verdict = if second.is_err() {
        "still waiting"
    } else {
        "ready"
    };
    println!("second ready() after 50 ms: {verdict}");

    println!("first request finishes: {:?}", first.await.unwrap());

    svc.ready().await.unwrap();
    println!("now ready again");
}
