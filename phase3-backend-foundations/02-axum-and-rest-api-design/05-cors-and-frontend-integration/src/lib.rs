//! Exercises for 3.2.5 — CORS and frontend integration.
//!
//! Two tiny JSON endpoints and the router are provided. The exercise is the
//! three functions that build a `tower_http::cors::CorsLayer`: everything
//! this lesson teaches lives in the layer wrapped around the router, not in
//! the router itself. `tests/cors_test.rs` checks all three, but every doc
//! comment below is the complete specification — you should never need to
//! open the tests to know what to build.

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

/// The permissive policy for local development. Never ship it.
///
/// - **Origins:** every origin. Responses to both preflights and ordinary
///   requests carry `access-control-allow-origin: *` — the literal wildcard,
///   whatever `Origin` the request sent.
/// - **Methods:** every method. A preflight response carries
///   `access-control-allow-methods: *`.
/// - **Request headers:** every header. A preflight response carries
///   `access-control-allow-headers: *`.
/// - **Credentials:** not allowed. No response carries an
///   `access-control-allow-credentials` header.
pub fn dev_cors() -> CorsLayer {
    todo!("a CorsLayer that allows any origin, any method and any request header, each as the `*` wildcard, and no credentials")
}

/// The locked-down policy for production: one frontend, and only what it
/// actually uses.
///
/// - **Origins:** exactly `allowed_origin`, compared byte for byte with the
///   request's `Origin` header. When they are equal, the response carries
///   `access-control-allow-origin` set to that same origin (never `*`). When
///   they differ, the response carries **no** `access-control-allow-origin`
///   header at all, and its status is unchanged: a preflight from an unknown
///   origin still gets `200 OK`, it just isn't vouched for.
/// - **Methods:** `GET` and `POST`. A preflight response's
///   `access-control-allow-methods` lists both, and no others.
/// - **Request headers:** `content-type` only. A preflight response's
///   `access-control-allow-headers` lists `content-type` and nothing else,
///   even when the preflight asked for more (for example `authorization`).
/// - **Credentials:** not allowed. No response carries an
///   `access-control-allow-credentials` header.
///
/// # Panics
///
/// If `allowed_origin` is not a valid header value. A misconfigured origin
/// should stop the process at startup, not quietly break every browser
/// client at request time.
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    todo!("a CorsLayer that vouches only for `allowed_origin`, allows GET and POST, allows only the content-type request header, and no credentials")
}

/// The "Build" exercise: [`prod_cors`], but for several frontends at once,
/// read from one comma-separated string — the shape an `ALLOWED_ORIGINS`
/// setting usually has, e.g. `"https://anime.example.com, http://localhost:5173"`.
///
/// - **Parsing:** split `allowed_origins` on `,`. Trim spaces around each
///   entry, and skip entries that are empty after trimming (so a trailing
///   comma is harmless). A string with no entries at all vouches for nobody.
/// - **Origins:** every entry, each compared byte for byte with the request's
///   `Origin` header. A request from any listed origin gets
///   `access-control-allow-origin` set to *its own* origin; a request from
///   any other origin gets no `access-control-allow-origin` header and an
///   unchanged status.
/// - **Methods, request headers, credentials:** exactly as [`prod_cors`]:
///   `GET` and `POST`, `content-type` only, no credentials.
///
/// # Panics
///
/// - If an entry ends with `/`. A browser's `Origin` header never does, so
///   that entry could never match anything. The panic message must contain
///   the offending entry exactly as it appears after trimming.
/// - If an entry is not a valid header value.
pub fn prod_cors_from_list(allowed_origins: &str) -> CorsLayer {
    todo!("like prod_cors, but vouching for every origin in the comma-separated list, refusing entries that end with a slash")
}

/// Provided — the router, plus one line: `.layer(cors)`. Which `CorsLayer`
/// gets passed in is the caller's (and the tests') choice; the router itself
/// doesn't know dev from prod.
pub fn app(cors: CorsLayer) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .layer(cors)
}
