//! A handler that takes `Extension<AuthUser>` on a route nobody put behind
//! `require_auth`. It compiles and it runs. Nothing says the extension was
//! never inserted until a request arrives.

use axum::body::{to_bytes, Body};
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use p3_07_03_jwt_and_tower_middleware::whoami;
use tower::ServiceExt;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/whoami", get(whoami));

    let request = Request::builder()
        .uri("/whoami")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    println!("status: {}", response.status());
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("body:   {}", String::from_utf8_lossy(&body));
}
