//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build --example 07-error-not-a-response-broken --features broken`
//! and read the error.

use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;

// A bare error type. It does not implement `IntoResponse`, so `axum` has no
// way to turn the `Err` side into the 401 the client should see.
struct NotAllowed;

async fn gate(request: Request, next: Next) -> Result<Response, NotAllowed> {
    if request.headers().contains_key("authorization") {
        Ok(next.run(request).await)
    } else {
        Err(NotAllowed)
    }
}

fn main() {
    let _router: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .route_layer(from_fn(gate));
}
