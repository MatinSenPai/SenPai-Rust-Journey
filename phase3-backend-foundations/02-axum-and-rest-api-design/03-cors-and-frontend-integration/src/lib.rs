//! Two tiny JSON endpoints (provided) plus the actual exercise: two
//! functions returning a configured `tower_http::cors::CorsLayer`. The
//! router is deliberately boring — everything this lesson teaches lives in
//! the layer wrapped around it.

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

/// Provided — a static list, no store, no state. The lesson is the CORS
/// layer, not the handlers.
async fn list_anime() -> Json<Value> {
    Json(json!([
        { "id": 1, "title": "Frieren", "rating": 9 },
        { "id": 2, "title": "Mob Psycho 100", "rating": 10 },
    ]))
}

/// Provided — echoes the JSON body back with `201`. Its real job here: a
/// `POST` with `content-type: application/json` is exactly the kind of
/// request a browser refuses to send cross-origin without a successful
/// preflight first.
async fn create_anime(Json(input): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(input))
}

/// Permissive CORS for local development: any origin, any method, any
/// header. Never ship this — see `prod_cors` and the README.
pub fn dev_cors() -> CorsLayer {
    todo!(
        "CorsLayer::new().allow_origin(tower_http::cors::Any)\
         .allow_methods(tower_http::cors::Any).allow_headers(tower_http::cors::Any) — `Any` is \
         a unit struct tower-http provides exactly for this, and it becomes a literal `*` in \
         the response headers (CorsLayer::permissive() is the same thing, prebuilt)"
    )
}

/// Locked-down CORS for production: exactly one allowed origin, and only
/// the methods and headers the real frontend actually uses.
///
/// Panics if `allowed_origin` isn't a valid header value — a misconfigured
/// origin should kill the process at startup, not silently break every
/// browser client at request time.
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    todo!(
        "parse the origin first: let origin = allowed_origin.parse::<axum::http::HeaderValue>()\
         .expect(\"invalid allowed origin\"); then CorsLayer::new()\
         .allow_origin(tower_http::cors::AllowOrigin::list([origin]))\
         .allow_methods([axum::http::Method::GET, axum::http::Method::POST])\
         .allow_headers([axum::http::header::CONTENT_TYPE]). \
         Use AllowOrigin::list([origin]), not a bare .allow_origin(origin): the list form makes \
         the server check the request's Origin and send the allow-origin header only when it \
         matches, so an unknown origin is declined server-side instead of relying on the browser"
    )
}

/// Provided — the same router shape as the last two lessons plus one new
/// line: `.layer(cors)`. Which `CorsLayer` gets passed in is the caller's
/// (and the tests') choice — the router itself doesn't know dev from prod.
pub fn app(cors: CorsLayer) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .layer(cors)
}
