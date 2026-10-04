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

/// Permissive CORS for local development. Never ship it.
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Locked-down CORS for production: one frontend, only what it uses.
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    prod_layer(vec![allowed_origin
        .parse::<HeaderValue>()
        .expect("invalid allowed origin")])
}

/// Like `prod_cors`, for a comma-separated list of origins.
pub fn prod_cors_from_list(allowed_origins: &str) -> CorsLayer {
    let origins = allowed_origins
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            assert!(
                !entry.ends_with('/'),
                "allowed origin {entry:?} ends with a slash and can never match"
            );
            entry
                .parse::<HeaderValue>()
                .expect("invalid allowed origin")
        })
        .collect();
    prod_layer(origins)
}

fn prod_layer(origins: Vec<HeaderValue>) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}

pub fn app(cors: CorsLayer) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .layer(cors)
}
