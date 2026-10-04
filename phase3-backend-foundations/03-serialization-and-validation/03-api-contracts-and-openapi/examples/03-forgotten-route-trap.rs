//! Compiles, runs, and is wrong: `/health` works but the contract never
//! mentions it, and nothing in the compiler objects.

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;
use utoipa::OpenApi;

#[utoipa::path(get, path = "/ping", responses((status = 200, description = "pong")))]
async fn ping() -> &'static str {
    "pong"
}

// Added later by someone in a hurry: no `#[utoipa::path]`, not in `paths(...)`.
async fn health() -> &'static str {
    "ok"
}

#[derive(OpenApi)]
#[openapi(paths(ping))]
struct Doc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/health", get(health));
    let doc = serde_json::to_value(Doc::openapi()).unwrap();

    for path in ["/ping", "/health"] {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let status = app.clone().oneshot(request).await.unwrap().status();
        let documented = !doc["paths"][path].is_null();
        println!("GET {path:<7} -> {status}   documented: {documented}");
    }
}
