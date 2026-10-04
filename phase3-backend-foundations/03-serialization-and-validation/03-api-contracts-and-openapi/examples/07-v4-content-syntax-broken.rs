//! DELIBERATELY BROKEN — expected: a macro parse error (`expected `,``), no error code
//! Run `cargo run -p p3-03-03-api-contracts-and-openapi --features broken --example 07-v4-content-syntax-broken`
//! and read the error: this is the `utoipa` 4 spelling of `content(...)`.

use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/ping",
    responses((status = 200, content(("text/plain" = String))))
)]
async fn ping() -> &'static str {
    "pong"
}

#[derive(OpenApi)]
#[openapi(paths(ping))]
struct Doc;

fn main() {
    println!("{}", Doc::openapi().to_pretty_json().unwrap());
}
