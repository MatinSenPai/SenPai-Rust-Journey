//! DELIBERATELY BROKEN — expected: a run-time panic when the router is built
//!
//! `/:id` was the path-parameter syntax up to `axum` 0.7. In 0.8 a path
//! parameter is written `/{id}`, and `.route` refuses the old spelling.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 07-colon-path-syntax-broken --features broken

use axum::extract::Path;
use axum::routing::get;
use axum::Router;

async fn anime(Path(id): Path<u32>) -> String {
    format!("anime #{id}")
}

fn main() {
    println!("building the router...");
    let _app: Router = Router::new().route("/anime/:id", get(anime));
    println!("router built");
}
