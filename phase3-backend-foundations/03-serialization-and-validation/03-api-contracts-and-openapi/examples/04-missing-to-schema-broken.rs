//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo run -p p3-03-03-api-contracts-and-openapi --features broken --example 04-missing-to-schema-broken`
//! and read the error: `Genre` is named in `body = ...` but never derives `ToSchema`.

use axum::Json;
use serde::Serialize;
use utoipa::OpenApi;

#[derive(Serialize)]
struct Genre {
    name: String,
}

#[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
async fn genre() -> Json<Genre> {
    Json(Genre {
        name: "isekai".into(),
    })
}

#[derive(OpenApi)]
#[openapi(paths(genre))]
struct Doc;

fn main() {
    println!("{}", Doc::openapi().to_pretty_json().unwrap());
}
