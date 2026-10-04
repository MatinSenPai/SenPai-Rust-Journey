//! DELIBERATELY BROKEN — expected: E0425
//! Run `cargo run -p p3-03-03-api-contracts-and-openapi --features broken --example 05-path-without-attribute-broken`
//! and read the error: `paths(health)` names a function with no `#[utoipa::path]`.

use utoipa::OpenApi;

async fn health() -> &'static str {
    "ok"
}

#[derive(OpenApi)]
#[openapi(paths(health))]
struct Doc;

fn main() {
    println!("{}", Doc::openapi().to_pretty_json().unwrap());
}
