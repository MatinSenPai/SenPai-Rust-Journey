//! Compiles, runs, and is wrong: the contract promises `201`, the handler
//! (a bare `Json`) answers `200`. `utoipa` documents what you write in the
//! attribute; it never looks at what the function body does.

use axum::body::Body;
use axum::http::Request;
use axum::routing::post;
use axum::{Json, Router};
use tower::ServiceExt;
use utoipa::OpenApi;

#[utoipa::path(post, path = "/anime", responses((status = 201, description = "created")))]
async fn create() -> Json<&'static str> {
    Json("Frieren")
}

#[derive(OpenApi)]
#[openapi(paths(create))]
struct Doc;

#[tokio::main]
async fn main() {
    let doc = serde_json::to_value(Doc::openapi()).unwrap();
    let promised: Vec<&String> = doc["paths"]["/anime"]["post"]["responses"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    println!("contract says POST /anime answers {promised:?}");

    let app = Router::new().route("/anime", post(create));
    let request = Request::post("/anime").body(Body::empty()).unwrap();
    let status = app.oneshot(request).await.unwrap().status();
    println!("the handler answers {status}");
}
