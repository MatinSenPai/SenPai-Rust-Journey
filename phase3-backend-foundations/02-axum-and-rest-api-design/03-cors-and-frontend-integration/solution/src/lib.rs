use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

async fn list_anime() -> Json<Value> {
    Json(json!([
        { "id": 1, "title": "Frieren", "rating": 9 },
        { "id": 2, "title": "Mob Psycho 100", "rating": 10 },
    ]))
}

async fn create_anime(Json(input): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(input))
}

/// Permissive CORS for local development: any origin, any method, any
/// header. Never ship this — see `prod_cors`.
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Locked-down CORS for production: exactly one allowed origin, and only
/// the methods and headers the real frontend actually uses.
///
/// Panics if `allowed_origin` isn't a valid header value — a misconfigured
/// origin should kill the process at startup, not silently break every
/// browser client at request time.
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    let origin = allowed_origin
        .parse::<HeaderValue>()
        .expect("invalid allowed origin");

    // `AllowOrigin::list([origin])` — not a bare `.allow_origin(origin)` — so
    // the server *validates* the request's Origin against the allow-list and
    // echoes the header only on a match. A bare single origin would instead
    // stamp that fixed value on every response (even for evil.example.com) and
    // lean entirely on the browser to reject it. Validating server-side too is
    // defense in depth, and it's what the "unknown origin isn't vouched for"
    // test pins down.
    CorsLayer::new()
        .allow_origin(AllowOrigin::list([origin]))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}

pub fn app(cors: CorsLayer) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .layer(cors)
}
