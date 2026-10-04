//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build --example 06-wrong-state-type-broken --features broken`
//! and read the error.

use axum::extract::{Request, State};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use p3_07_03_jwt_and_tower_middleware::JwtConfig;

// The middleware asks for a `String` as its state...
async fn check(State(_secret): State<String>, request: Request, next: Next) -> Response {
    next.run(request).await
}

fn main() {
    // ...but the layer is given a `JwtConfig`.
    let config = JwtConfig::new("demo-secret-for-3-7-3");
    let _router: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .route_layer(from_fn_with_state(config, check));
}
