//! DELIBERATELY BROKEN — expected: a run-time panic when the layer is applied
//! to the router.
//!
//! "Allow cookies, from any website" is the one CORS policy the spec forbids
//! outright. tower-http refuses to build it: the panic happens while the app
//! is being put together, before a single request arrives.
//!
//!     cargo run -p p3-02-05-cors-and-frontend-integration --example 04-credentials-with-wildcard-broken --features broken

use tower_http::cors::{Any, CorsLayer};

use p3_02_05_cors_and_frontend_integration::app;

fn main() {
    let cors = CorsLayer::new().allow_origin(Any).allow_credentials(true);

    println!("layer built, applying it to the router...");
    let _router = app(cors);
    println!("router built");
}
